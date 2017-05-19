
extern crate num_cpus;
#[macro_use]
extern crate error_chain;

extern crate rayon;

use rayon::prelude::*;

use std::fs::File;
use std::io::{BufReader, BufRead};

use std::collections::HashMap;
use std::time::Instant;

error_chain! {
    foreign_links {
        Io(std::io::Error);
    }
}

type Possibility = Vec<u8>;
type Grid = [Option<u8>; 81];

fn print_grid(g: Grid) {

    let mut cnt = 0;
    let mut line = 0;

    for &x in g.iter() {

        cnt += 1;

        match x {
            Some(i) => print!("{}", i),
            None => print!("_"),
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

fn check_no_redundant_value(g: Grid, val: [usize; 9]) -> bool {
    let mut checked: [bool; 9] = [false; 9];
    for &v in &val {
        let valv = g[v as usize];
        if valv != None {
            // print_grid(g);
            // println!("{}", valv.unwrap());
            if checked[(valv.unwrap() - 1) as usize] {
                return false;
            }
            checked[(valv.unwrap() - 1) as usize] = true;
        }
    }
    true
}

fn get_line(index: usize) -> usize {
    (index / 9) * 9
}

fn get_column(index: usize) -> usize {
    index % 9
}

fn get_head_of_block(index: usize) -> usize {
    index - (index % 3) - (index / 9 % 3) * 9
}

fn check_grid_at(g: Grid, index: usize) -> bool {

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

fn is_grid_complete(g: Grid) -> bool {
    g.iter().all(|&x| x != None)
}

fn clone_grid(g: Grid) -> Grid {
    let mut new_g: Grid = [None; 81];
    for x in 0..g.len() {
        new_g[x] = g[x];
    }
    new_g
}

fn build_possible_values_grid(grid: Grid) -> HashMap<usize, Possibility> {
    let mut possible_grid = HashMap::new();

    for (index, item) in grid.iter().enumerate() {
        if item.is_none() {
            let possible_values = get_possible_values(grid, index);
            let len = possible_values.len();
            possible_grid.insert(index, possible_values);
            if len == 0 {
                return possible_grid;
            }
        }
    }
    possible_grid
}

fn get_possible_values(grid: Grid, index: usize) -> Vec<u8> {

    let column = get_column(index);
    let head_of_line = get_line(index);
    let head_of_block = get_head_of_block(index);
    let values = [column,
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
                  head_of_block + 20];

    let mut values_again = [false; 9];
    let mut n_found = 0;
    let mut val_found: Vec<u8> = vec![];

    for &val in &values {
        if let Some(num) = grid[val] {
            if !values_again[num as usize - 1] {
                values_again[num as usize - 1] = true;
                n_found += 1;
                if n_found == 9 {
                    return val_found;
                }
            }
        }
    }

    for (idx, val) in values_again.iter().enumerate() {
        if !val {
            val_found.push(idx as u8 + 1);
        }
    }

    val_found
}

fn solve_grid(grid: Grid) -> Option<Grid> {

    let possible_values_grid = build_possible_values_grid(grid);
    let length = possible_values_grid.len();

    if length > 0 {
        let (&index, possible_values) = possible_values_grid.iter()
            .min_by_key(|x| x.1.len())
            .unwrap();

        match possible_values.len() {
            0 => None,
            1 => {
                // fill in all the cells where there is only one possiblitiy
                let mut new_g = clone_grid(grid);

                for (&index, possible_values) in &possible_values_grid {
                    if possible_values.len() == 1 {
                        new_g[index] = Some(possible_values[0]);
                    }
                }

                solve_grid(new_g)
            }
            _ => {

                let mut new_g = clone_grid(grid);

                for val in possible_values {
                    new_g[index] = Some(*val);

                    if check_grid_at(new_g, index) {
                        let gx = solve_grid(new_g);
                        if gx.is_some() {
                            return gx;
                        }
                    }
                }
                None
            }
        }

    } else if is_grid_complete(grid) {
        Some(grid)
    } else {
        None
    }
}

fn parse_grid(grid_string: &str) -> Grid {
    let mut grid = [None; 81];

    let mut i = 0;
    for splitted in grid_string.split_whitespace() {
        for s in splitted.split("") {
            match s {
                "" => {}
                "_" => {
                    i += 1;
                }
                val => {
                    grid[i] = Some(val.parse::<u8>().unwrap());
                    i += 1;
                }
            }
        }
    }

    grid
}

fn run() -> Result<()> {

    let path = "grids.txt";
    let input = File::open(path)?;
    let buffered = BufReader::new(input);

    let mut line_buffer = vec![];
    for line in buffered.lines() {
        let line_content = line?;
        if line_content.len() > 0 {
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

    grid_strings.par_iter().for_each(|grid_string_local| treat_grid(grid_string_local));

    Ok(())
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

quick_main!(run);

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
    assert_eq!(vec![2, 4, 5, 6], get_possible_values(grid, 1));

    // /
    // let mut grid: Grid = [None; 81];
    // for i in 0..81 {
    // grid[i] = Some(i);
    // }
    // print_grid(grid);
    // assert!(false);
    //
}

// extern crate test;
// use test::Bencher;
//
// #[bench]
// fn benchmark(b: &mut Bencher) {
//
// let grid_string = r#"
// _ _ _   _ _ _   _ _ _
// _ 1 _   6 _ _   _ _ 8
// _ _ 5   _ 7 _   _ 1 _
//
// _ _ 8   _ _ _   _ _ _
// _ _ _   4 1 9   _ _ _
// _ _ _   _ _ _   2 _ _
//
// _ 5 _   _ 3 _   7 _ _
// 4 _ _   _ _ 8   _ 9 _
// _ _ _   _ _ _   _ _ _"#;
//
// let grid: Grid = parse_grid(grid_string);
//
// b.iter(||
// solve_grid(grid)
// );
// }
//
