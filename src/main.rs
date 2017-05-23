
#![feature(test)]

extern crate num_cpus;
#[macro_use]
extern crate error_chain;

extern crate rayon;

use rayon::prelude::*;

use std::fs::File;
use std::io::{BufReader, BufRead};

use std::time::Instant;

error_chain! {
    foreign_links {
        Io(std::io::Error);
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum CellValue {
    Value(u8),
    Possibilities([bool; 9]),
}

type Grid = [CellValue; 81];

impl CellValue {
    pub fn is_value(&self) -> bool {
        match *self {
            CellValue::Value(_) => true,
            CellValue::Possibilities(_) => false,
        }
    }

    pub fn get_nb_possibility(&self) -> u8 {
        match *self {
            CellValue::Value(_) => 10,
            CellValue::Possibilities(values) => {
                values.iter().fold(0, |acc, &p| if p { acc + 1 } else { acc })
            }
        }
    }
}

fn is_grid_complete(g: Grid) -> bool {
    g.iter().all(|&x| x.is_value())
}

fn clone_grid(g: Grid) -> Grid {
    let mut new_g: Grid = [CellValue::Possibilities([true; 9]); 81];
    for x in 0..g.len() {
        new_g[x] = g[x];
    }
    new_g
}

fn print_grid(g: Grid) {
    print_grid_option(g, false);
}

fn print_grid_option(g: Grid, with_possibilities: bool) {

    let mut cnt = 0;
    let mut line = 0;

    for &x in g.iter() {

        cnt += 1;

        match x {
            CellValue::Value(i) => print!("{}", i),
            CellValue::Possibilities(p) => {
                if with_possibilities {
                    print!("(");
                    for (idx, &val) in p.iter().enumerate() {
                        if val {
                            print!("{},", idx + 1);
                        }
                    }
                    print!(")");
                } else {
                    print!("_");
                }
            }
        }

        if cnt == 9 {
            line += 1;
            println!("");
            cnt = 0;
            if line == 3 {
                line = 0;
                println!("");
            }
        } else if cnt % 3 == 0 {
            print!("   ");

        } else {
            print!(" ");
        }
    }
}

fn get_line(index: u8) -> u8 {
    (index / 9) * 9
}

fn get_column(index: u8) -> u8 {
    index % 9
}

fn get_head_of_block(index: u8) -> u8 {
    index - (index % 3) - (index / 9 % 3) * 9
}

fn check_grid_at(g: Grid, index: u8) -> bool {

    let line = get_line(index);

    // check lines
    let val = [line, line + 1, line + 2, line + 3, line + 4, line + 5, line + 6, line + 7,
               line + 8];
    if !check_no_redundant_value(g, val) {
        return false;
    }

    // check column
    let column = get_column(index);
    let val = [column,
               column + 9,
               column + 18,
               column + 27,
               column + 36,
               column + 45,
               column + 54,
               column + 63,
               column + 72];
    if !check_no_redundant_value(g, val) {
        return false;
    }

    // check block constraint
    let head_of_block = get_head_of_block(index);
    let val = [head_of_block,
               head_of_block + 1,
               head_of_block + 2,
               head_of_block + 9,
               head_of_block + 10,
               head_of_block + 11,
               head_of_block + 18,
               head_of_block + 19,
               head_of_block + 20];
    if !check_no_redundant_value(g, val) {
        return false;
    }

    true
}

fn check_no_redundant_value(grid: Grid, val: [u8; 9]) -> bool {
    let mut checked: [bool; 9] = [false; 9];
    for &v in &val {
        if let CellValue::Value(cell_value) = grid[v as usize] {
            if checked[(cell_value - 1) as usize] {
                return false;
            }
            checked[(cell_value - 1) as usize] = true;
        }
    }
    true
}

fn get_adjacent_cells(index: u8) -> [u8; 27] {

    let column = get_column(index);
    let head_of_line = get_line(index);
    let head_of_block = get_head_of_block(index);
    [column,
     column + 9,
     column + 18,
     column + 27,
     column + 36,
     column + 45,
     column + 54,
     column + 63,
     column + 72,
     head_of_line,
     head_of_line + 1,
     head_of_line + 2,
     head_of_line + 3,
     head_of_line + 4,
     head_of_line + 5,
     head_of_line + 6,
     head_of_line + 7,
     head_of_line + 8,
     head_of_block,
     head_of_block + 1,
     head_of_block + 2,
     head_of_block + 9,
     head_of_block + 10,
     head_of_block + 11,
     head_of_block + 18,
     head_of_block + 19,
     head_of_block + 20]
}

fn complete_grid_at_value(mut grid: &mut Grid, val: [u8; 9]) -> Option<bool> {
    let mut checked: [bool; 9] = [true; 9];
    let mut poss_filled: [bool; 9] = [true; 9];
    let mut nb_values_filled = 0;
    for (i, &v) in val.iter().enumerate() {
        if let CellValue::Value(cell_value) = grid[v as usize] {
            nb_values_filled += 1;
            checked[(cell_value - 1) as usize] = false;
            poss_filled[i] = false;
        }
    }
    // Only one value is possible for a cell
    if nb_values_filled == 8 {
        for (poss_idx, &poss_val) in poss_filled.iter().enumerate() {
            if poss_val {
                for (check_idx, &checked_val) in checked.iter().enumerate() {
                    if checked_val {
                        grid[val[poss_idx] as usize] = CellValue::Value(check_idx as u8 + 1);
                    }
                }
            }
        }
        return Some(true);
    }
    Some(false)
}

fn complete_grid_at(mut grid: &mut Grid, index: u8) -> Option<bool> {

    let mut ret = false;

    let line = index * 9;

    // complete lines
    let val = [line, line + 1, line + 2, line + 3, line + 4, line + 5, line + 6, line + 7,
               line + 8];
    match complete_grid_at_value(grid, val) {
        Some(v) => {
            ret = v;
        }
        None => {
            return None;
        }
    }

    // complete column
    let column = index;
    let val = [column,
               column + 9,
               column + 18,
               column + 27,
               column + 36,
               column + 45,
               column + 54,
               column + 63,
               column + 72];
    match complete_grid_at_value(grid, val) {
        Some(v) => {
            ret = v;
        }
        None => {
            return None;
        }
    }

    // complete block
    let head_of_block = (index / 3 * 27) + (index % 3) * 3;
    let val = [head_of_block,
               head_of_block + 1,
               head_of_block + 2,
               head_of_block + 9,
               head_of_block + 10,
               head_of_block + 11,
               head_of_block + 18,
               head_of_block + 19,
               head_of_block + 20];
    match complete_grid_at_value(grid, val) {
        Some(v) => {
            ret = v;
        }
        None => {
            return None;
        }
    }

    Some(ret)
}

fn build_possible_values_grid(mut grid: &mut Grid) -> Option<bool> {

    let mut val_found = false;

    for index in 0..81 {
        let is_possibility = !grid[index].is_value();
        if is_possibility {
            let possible_value = get_possible_values(*grid, index as u8);
            match possible_value {
                Some(CellValue::Possibilities(_)) => {
                    grid[index] = possible_value.unwrap();
                }
                Some(CellValue::Value(_)) => {
                    grid[index] = possible_value.unwrap();
                    val_found = true;
                }
                None => {
                    return None;
                }
            }
        }
    }

    Some(val_found)
}

fn try_complete_grid(mut grid: &mut Grid) -> Option<bool> {
    let mut val_found = Some(false);
    for index in 0..9 {
        match complete_grid_at(grid, index) {
            Some(true) => {
                val_found = Some(true);
            }
            None => {
                return None;
            }
            _ => {}
        }
    }
    val_found
}


fn get_possible_values(grid: Grid, index: u8) -> Option<CellValue> {

    let values = get_adjacent_cells(index);
    let mut poss = CellValue::Possibilities([true; 9]);
    let mut n_found = 0;

    match poss {
        CellValue::Possibilities(ref mut values_again) => {
            for &val in &values {
                if let CellValue::Value(num) = grid[val as usize] {
                    if values_again[num as usize - 1] {
                        values_again[num as usize - 1] = false;
                        n_found += 1;
                        if n_found == 9 {
                            return None;
                        }
                    }
                }
            }

            // There is only one option left
            if n_found == 8 {
                for (idx, &value) in values_again.iter().enumerate() {
                    if value {
                        let new_cell_value = idx as u8 + 1;
                        return Some(CellValue::Value(new_cell_value));
                    }
                }
            }
        }
        _ => {}
    }

    Some(poss)
}

fn solve_grid(mut grid: Grid) -> Option<Grid> {

    try_complete_grid(&mut grid);
    let mut values_found = Some(true);

    while values_found == Some(true) {
        values_found = build_possible_values_grid(&mut grid);
        if values_found.is_some() {
            match try_complete_grid(&mut grid) {
                Some(true) => {
                    values_found = Some(true);
                }
                None => {
                    values_found = None;
                }
                _ => {}
            }
        }
    }
    if values_found.is_none() {
        return None;
    }

    // let mut values = [[false; 81];9];
    // let mut known_cells = [0; 9];
    // let mut options_pervalue = [0; 9];
    // for (idx, &value) in grid.iter().enumerate() {
    // match value {
    // CellValue::Possibilities(poss) => {
    // for (i, &p) in poss.iter().enumerate() {
    // if p {
    // values[i][idx] = true;
    // options_pervalue[i] += 1;
    // }
    // }
    // },
    // CellValue::Value(val) => {
    // known_cells[val as usize - 1] += 1;
    // }
    // }
    // }
    // println!("{:?}", known_cells);
    //
    // let option = known_cells.iter().enumerate()
    //    .filter(|v| *v.1 != 9).max_by_key(|v| v.1);
    //
    // let option = options_pervalue.iter().enumerate().filter(|v| *v.1 > 0).min_by_key(|v| v.1);
    //
    // start by the number with the biggest numbers of values already in the grid when guessing
    // let res = grid.iter().enumerate().min_by_key(|val| val.1.get_nb_possibility());
    //
    //
    // Complete line
    // Complete column
    // Complete cell block
    // set values with the least options
    // fill the cell whose values have the least cell options
    //
    // match option {
    // Some(val) => {
    // match res {
    // Some((index, &cell)) => {
    //
    // if *val.1 < cell.get_nb_possibility() {
    //
    // let cell_value = val.0;
    // let res1 = values.iter().enumerate().
    // filter(|val| val.1.len() > 0).min_by_key(|val| val.1.len());
    //
    // println!("{:?}", values[cell_value]);
    // print_grid_option(grid, true);
    // for (index ,&val) in values[cell_value].iter().enumerate() {
    // if val {
    // let mut new_g = clone_grid(grid);
    // new_g[index as usize] = CellValue::Value(cell_value as u8 + 1);
    //
    // if check_grid_at(new_g, index as u8) {
    // let gx = solve_grid(new_g);
    // if gx.is_some() {
    // return gx;
    // }
    // }
    // }
    // }
    // } else {
    // match cell {
    // CellValue::Value(_) => {}
    // CellValue::Possibilities(poss) => {
    // let mut new_g = clone_grid(grid);
    //
    // for (idx, &val) in poss.iter().enumerate() {
    // if val {
    // new_g[index as usize] = CellValue::Value(idx as u8 + 1);
    //
    // if check_grid_at(new_g, index as u8) {
    // let gx = solve_grid(new_g);
    // if gx.is_some() {
    // return gx;
    // }
    // }
    // }
    // }
    // return None;
    // }
    // }
    // }
    // },
    // None => {
    // println!("res was none");
    // }
    // }
    // },
    // None => {
    // println!("option was none");
    // },
    // }
    //

    // start by the number with the biggest numbers of values already in the grid when guessing
    let res = grid.iter().enumerate().min_by_key(|val| val.1.get_nb_possibility());

    match res {
        Some((index, &cell)) => {
            match cell {
                CellValue::Value(_) => {}
                CellValue::Possibilities(poss) => {
                    let mut new_g = clone_grid(grid);

                    for (idx, &val) in poss.iter().enumerate() {
                        if val {
                            new_g[index as usize] = CellValue::Value(idx as u8 + 1);

                            if check_grid_at(new_g, index as u8) {
                                let gx = solve_grid(new_g);
                                if gx.is_some() {
                                    return gx;
                                }
                            }
                        }
                    }
                    return None;
                }
            }
        }
        None => {
            return None;
        }
    }


    if is_grid_complete(grid) {
        Some(grid)
    } else {
        // println!("{:?}", res);
        // println!("{:?}", option);
        // print_grid_option(grid, true);
        //
        None
    }
}


fn parse_grid(grid_string: &str) -> Grid {
    let mut grid = [CellValue::Possibilities([true; 9]); 81];

    let mut i = 0;
    for splitted in grid_string.split_whitespace() {
        for s in splitted.split("") {
            match s {
                "" => {}
                "_" => {
                    i += 1;
                }
                val => {
                    grid[i] = CellValue::Value(val.parse::<u8>().unwrap());
                    i += 1;
                }
            }
        }
    }

    grid
}

fn treat_grid(grid_string: &str) {

    let grid: Grid = parse_grid(grid_string);

    let now = Instant::now();
    let new_grid = solve_grid(grid);
    let duration = now.elapsed();

    match new_grid {
        Some(new_grid) => {
            println!("Grid complete ! in {} ms",
                     (1000_000_000 * duration.as_secs() + duration.subsec_nanos() as u64) /
                     (1000_000));
            print_grid(grid);
            print_grid(new_grid);
        }
        None => {
            println!("Couldn't solve the sudoku :( in {} ms",
                     (1000_000_000 * duration.as_secs() + duration.subsec_nanos() as u64) /
                     (1000_000));
            print_grid(grid);
        }
    }
}

fn run() -> Result<()> {

    let path = "grids.txt";
    let input = File::open(path)?;
    let buffered = BufReader::new(input);

    let mut line_buffer = vec![];
    for line in buffered.lines() {
        let line_content = line?;
        if !line_content.is_empty() {
            line_buffer.push(line_content);
        }
    }

    let mut grid_strings = vec![];
    let mut grid_string = String::from("");
    for (index, line) in line_buffer.iter().enumerate() {
        if index % 9 == 0 {
            if index > 0 {
                grid_strings.push(grid_string.clone());
            }
            grid_string = String::from("");
        }
        grid_string.push_str(line);
    }
    grid_strings.push(grid_string);

    grid_strings.par_iter().for_each(|grid_string_local| treat_grid(grid_string_local));

    Ok(())
}

quick_main!(run);

#[cfg(test)]
#[test]
fn test() {
    assert!(get_head_of_block(1) == 0);
    assert!(get_head_of_block(11) == 0);
    assert!(get_head_of_block(70) == 60);
    assert!(get_head_of_block(78) == 60);
    assert!(get_head_of_block(54) == 54);
    assert!(get_head_of_block(56) == 54);
    assert!(get_head_of_block(49) == 30);
    assert!(get_head_of_block(47) == 27);
    assert!(get_head_of_block(28) == 27);
    assert!(get_head_of_block(35) == 33);
    assert!(get_head_of_block(26) == 6);
    assert!(get_head_of_block(32) == 30);
    assert!(get_head_of_block(17) == 6);
}

#[cfg(test)]
#[test]
fn test_get_possible_values() {
    let grid_string = r#"
        1 _ _   _ 3 _   _ _ 9
        _ _ _   _ 2 _   _ _ _
        3 _ 7   _ _ _   _ 5 _

        _ _ _   _ _ _   _ _ _
        _ _ 4   _ _ _   _ _ _
        7 _ _   _ _ _   _ _ _

        _ _ _   _ _ _   _ _ _
        _ _ _   _ _ _   _ _ _
        _ 8 _   _ _ _   _ _ _"#;

    let grid: Grid = parse_grid(grid_string);

    println!("{:?}", get_possible_values(grid, 1));
    // assert_eq!(vec![2, 4, 5, 6], get_possible_values(grid, 1));
    assert_eq!(Some(CellValue::Possibilities([false, true, false, true, true, true, false,
                                              false, false])),
               get_possible_values(grid, 1));
    assert_eq!(Some(CellValue::Possibilities([false, true, false, true, false, true, true,
                                              true, false])),
               get_possible_values(grid, 8));
}

#[cfg(test)]
#[test]
fn test_print_grid() {
    let grid_string = r#"
        1 _ _   _ 3 _   _ _ 9
        _ _ _   _ 2 _   _ _ _
        3 _ 7   _ _ _   _ 5 _

        _ _ _   _ _ _   _ _ _
        _ _ 4   _ _ _   _ _ _
        7 _ _   _ _ _   _ _ _

        _ _ _   _ _ _   _ _ _
        _ _ _   _ _ _   _ _ _
        _ 8 _   _ _ _   _ _ _"#;

    let grid: Grid = parse_grid(grid_string);

    print_grid(grid);
}

extern crate test;
#[cfg(test)]
use test::Bencher;

#[cfg(test)]
#[bench]
fn benchmark(b: &mut Bencher) {

    let grid_string = r#"
_ _ _   _ _ _   _ _ _
_ 1 _   6 _ _   _ _ 8
_ _ 5   _ 7 _   _ 1 _

_ _ 8   _ _ _   _ _ _
_ _ _   4 1 9   _ _ _
_ _ _   _ _ _   2 _ _

_ 5 _   _ 3 _   7 _ _
4 _ _   _ _ 8   _ 9 _
_ _ _   _ _ _   _ _ _"#;

    let grid: Grid = parse_grid(grid_string);

    b.iter(|| solve_grid(grid));
}

// #[cfg(test)]
// #[bench]
// fn benchmark2(b: &mut Bencher) {
//
// let grid_string = r#"
// _ _ _   _ _ _   _ _ _
// _ _ _   _ _ 3   _ 8 5
// _ _ 1   _ 2 _   _ _ _
//
// _ _ _   5 _ 7   _ _ _
// _ _ 4   _ _ _   1 _ _
// _ 9 _   _ _ _   _ _ _
//
// 5 _ _   _ _ _   _ 7 3
// _ _ 2   _ 1 _   _ _ _
// _ _ _   _ 4 _   _ _ 9"#;
//
// let grid: Grid = parse_grid(grid_string);
//
// b.iter(|| solve_grid(grid));
// }
//
