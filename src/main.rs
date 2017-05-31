
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
    g.iter().enumerate().all(|x| x.1.is_value())
}

fn is_grid_complete_full(g: Grid) -> bool {
    g.iter().enumerate().all(|x| x.1.is_value() && check_grid_at(g, x.0 as u8))
}

fn clone_grid(g: Grid) -> Grid {
    let mut new_g: Grid = [CellValue::Value(0); 81];
    for x in 0..81 {
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

static ADJACENT_VALUES: [[u8; 20]; 81] =
    [[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 18, 19, 20, 27, 36, 45, 54, 63, 72],
     [0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 18, 19, 20, 28, 37, 46, 55, 64, 73],
     [0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 18, 19, 20, 29, 38, 47, 56, 65, 74],
     [0, 1, 2, 4, 5, 6, 7, 8, 12, 13, 14, 21, 22, 23, 30, 39, 48, 57, 66, 75],
     [0, 1, 2, 3, 5, 6, 7, 8, 12, 13, 14, 21, 22, 23, 31, 40, 49, 58, 67, 76],
     [0, 1, 2, 3, 4, 6, 7, 8, 12, 13, 14, 21, 22, 23, 32, 41, 50, 59, 68, 77],
     [0, 1, 2, 3, 4, 5, 7, 8, 15, 16, 17, 24, 25, 26, 33, 42, 51, 60, 69, 78],
     [0, 1, 2, 3, 4, 5, 6, 8, 15, 16, 17, 24, 25, 26, 34, 43, 52, 61, 70, 79],
     [0, 1, 2, 3, 4, 5, 6, 7, 15, 16, 17, 24, 25, 26, 35, 44, 53, 62, 71, 80],
     [0, 1, 2, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 27, 36, 45, 54, 63, 72],
     [0, 1, 2, 9, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 28, 37, 46, 55, 64, 73],
     [0, 1, 2, 9, 10, 12, 13, 14, 15, 16, 17, 18, 19, 20, 29, 38, 47, 56, 65, 74],
     [3, 4, 5, 9, 10, 11, 13, 14, 15, 16, 17, 21, 22, 23, 30, 39, 48, 57, 66, 75],
     [3, 4, 5, 9, 10, 11, 12, 14, 15, 16, 17, 21, 22, 23, 31, 40, 49, 58, 67, 76],
     [3, 4, 5, 9, 10, 11, 12, 13, 15, 16, 17, 21, 22, 23, 32, 41, 50, 59, 68, 77],
     [6, 7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 24, 25, 26, 33, 42, 51, 60, 69, 78],
     [6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17, 24, 25, 26, 34, 43, 52, 61, 70, 79],
     [6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 24, 25, 26, 35, 44, 53, 62, 71, 80],
     [0, 1, 2, 9, 10, 11, 19, 20, 21, 22, 23, 24, 25, 26, 27, 36, 45, 54, 63, 72],
     [0, 1, 2, 9, 10, 11, 18, 20, 21, 22, 23, 24, 25, 26, 28, 37, 46, 55, 64, 73],
     [0, 1, 2, 9, 10, 11, 18, 19, 21, 22, 23, 24, 25, 26, 29, 38, 47, 56, 65, 74],
     [3, 4, 5, 12, 13, 14, 18, 19, 20, 22, 23, 24, 25, 26, 30, 39, 48, 57, 66, 75],
     [3, 4, 5, 12, 13, 14, 18, 19, 20, 21, 23, 24, 25, 26, 31, 40, 49, 58, 67, 76],
     [3, 4, 5, 12, 13, 14, 18, 19, 20, 21, 22, 24, 25, 26, 32, 41, 50, 59, 68, 77],
     [6, 7, 8, 15, 16, 17, 18, 19, 20, 21, 22, 23, 25, 26, 33, 42, 51, 60, 69, 78],
     [6, 7, 8, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 26, 34, 43, 52, 61, 70, 79],
     [6, 7, 8, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 35, 44, 53, 62, 71, 80],
     [0, 9, 18, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 45, 46, 47, 54, 63, 72],
     [1, 10, 19, 27, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 45, 46, 47, 55, 64, 73],
     [2, 11, 20, 27, 28, 30, 31, 32, 33, 34, 35, 36, 37, 38, 45, 46, 47, 56, 65, 74],
     [3, 12, 21, 27, 28, 29, 31, 32, 33, 34, 35, 39, 40, 41, 48, 49, 50, 57, 66, 75],
     [4, 13, 22, 27, 28, 29, 30, 32, 33, 34, 35, 39, 40, 41, 48, 49, 50, 58, 67, 76],
     [5, 14, 23, 27, 28, 29, 30, 31, 33, 34, 35, 39, 40, 41, 48, 49, 50, 59, 68, 77],
     [6, 15, 24, 27, 28, 29, 30, 31, 32, 34, 35, 42, 43, 44, 51, 52, 53, 60, 69, 78],
     [7, 16, 25, 27, 28, 29, 30, 31, 32, 33, 35, 42, 43, 44, 51, 52, 53, 61, 70, 79],
     [8, 17, 26, 27, 28, 29, 30, 31, 32, 33, 34, 42, 43, 44, 51, 52, 53, 62, 71, 80],
     [0, 9, 18, 27, 28, 29, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 54, 63, 72],
     [1, 10, 19, 27, 28, 29, 36, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 55, 64, 73],
     [2, 11, 20, 27, 28, 29, 36, 37, 39, 40, 41, 42, 43, 44, 45, 46, 47, 56, 65, 74],
     [3, 12, 21, 30, 31, 32, 36, 37, 38, 40, 41, 42, 43, 44, 48, 49, 50, 57, 66, 75],
     [4, 13, 22, 30, 31, 32, 36, 37, 38, 39, 41, 42, 43, 44, 48, 49, 50, 58, 67, 76],
     [5, 14, 23, 30, 31, 32, 36, 37, 38, 39, 40, 42, 43, 44, 48, 49, 50, 59, 68, 77],
     [6, 15, 24, 33, 34, 35, 36, 37, 38, 39, 40, 41, 43, 44, 51, 52, 53, 60, 69, 78],
     [7, 16, 25, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 44, 51, 52, 53, 61, 70, 79],
     [8, 17, 26, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 51, 52, 53, 62, 71, 80],
     [0, 9, 18, 27, 28, 29, 36, 37, 38, 46, 47, 48, 49, 50, 51, 52, 53, 54, 63, 72],
     [1, 10, 19, 27, 28, 29, 36, 37, 38, 45, 47, 48, 49, 50, 51, 52, 53, 55, 64, 73],
     [2, 11, 20, 27, 28, 29, 36, 37, 38, 45, 46, 48, 49, 50, 51, 52, 53, 56, 65, 74],
     [3, 12, 21, 30, 31, 32, 39, 40, 41, 45, 46, 47, 49, 50, 51, 52, 53, 57, 66, 75],
     [4, 13, 22, 30, 31, 32, 39, 40, 41, 45, 46, 47, 48, 50, 51, 52, 53, 58, 67, 76],
     [5, 14, 23, 30, 31, 32, 39, 40, 41, 45, 46, 47, 48, 49, 51, 52, 53, 59, 68, 77],
     [6, 15, 24, 33, 34, 35, 42, 43, 44, 45, 46, 47, 48, 49, 50, 52, 53, 60, 69, 78],
     [7, 16, 25, 33, 34, 35, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 53, 61, 70, 79],
     [8, 17, 26, 33, 34, 35, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 62, 71, 80],
     [0, 9, 18, 27, 36, 45, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 72, 73, 74],
     [1, 10, 19, 28, 37, 46, 54, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 72, 73, 74],
     [2, 11, 20, 29, 38, 47, 54, 55, 57, 58, 59, 60, 61, 62, 63, 64, 65, 72, 73, 74],
     [3, 12, 21, 30, 39, 48, 54, 55, 56, 58, 59, 60, 61, 62, 66, 67, 68, 75, 76, 77],
     [4, 13, 22, 31, 40, 49, 54, 55, 56, 57, 59, 60, 61, 62, 66, 67, 68, 75, 76, 77],
     [5, 14, 23, 32, 41, 50, 54, 55, 56, 57, 58, 60, 61, 62, 66, 67, 68, 75, 76, 77],
     [6, 15, 24, 33, 42, 51, 54, 55, 56, 57, 58, 59, 61, 62, 69, 70, 71, 78, 79, 80],
     [7, 16, 25, 34, 43, 52, 54, 55, 56, 57, 58, 59, 60, 62, 69, 70, 71, 78, 79, 80],
     [8, 17, 26, 35, 44, 53, 54, 55, 56, 57, 58, 59, 60, 61, 69, 70, 71, 78, 79, 80],
     [0, 9, 18, 27, 36, 45, 54, 55, 56, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74],
     [1, 10, 19, 28, 37, 46, 54, 55, 56, 63, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74],
     [2, 11, 20, 29, 38, 47, 54, 55, 56, 63, 64, 66, 67, 68, 69, 70, 71, 72, 73, 74],
     [3, 12, 21, 30, 39, 48, 57, 58, 59, 63, 64, 65, 67, 68, 69, 70, 71, 75, 76, 77],
     [4, 13, 22, 31, 40, 49, 57, 58, 59, 63, 64, 65, 66, 68, 69, 70, 71, 75, 76, 77],
     [5, 14, 23, 32, 41, 50, 57, 58, 59, 63, 64, 65, 66, 67, 69, 70, 71, 75, 76, 77],
     [6, 15, 24, 33, 42, 51, 60, 61, 62, 63, 64, 65, 66, 67, 68, 70, 71, 78, 79, 80],
     [7, 16, 25, 34, 43, 52, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 71, 78, 79, 80],
     [8, 17, 26, 35, 44, 53, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 78, 79, 80],
     [0, 9, 18, 27, 36, 45, 54, 55, 56, 63, 64, 65, 73, 74, 75, 76, 77, 78, 79, 80],
     [1, 10, 19, 28, 37, 46, 54, 55, 56, 63, 64, 65, 72, 74, 75, 76, 77, 78, 79, 80],
     [2, 11, 20, 29, 38, 47, 54, 55, 56, 63, 64, 65, 72, 73, 75, 76, 77, 78, 79, 80],
     [3, 12, 21, 30, 39, 48, 57, 58, 59, 66, 67, 68, 72, 73, 74, 76, 77, 78, 79, 80],
     [4, 13, 22, 31, 40, 49, 57, 58, 59, 66, 67, 68, 72, 73, 74, 75, 77, 78, 79, 80],
     [5, 14, 23, 32, 41, 50, 57, 58, 59, 66, 67, 68, 72, 73, 74, 75, 76, 78, 79, 80],
     [6, 15, 24, 33, 42, 51, 60, 61, 62, 69, 70, 71, 72, 73, 74, 75, 76, 77, 79, 80],
     [7, 16, 25, 34, 43, 52, 60, 61, 62, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 80],
     [8, 17, 26, 35, 44, 53, 60, 61, 62, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79]];

fn get_adjacent_cells(index: u8) -> [u8; 20] {
    // let column = get_column(index);
    // let head_of_line = get_line(index);
    // let head_of_block = get_head_of_block(index);
    // [column,
    // column + 9,
    // column + 18,
    // column + 27,
    // column + 36,
    // column + 45,
    // column + 54,
    // column + 63,
    // column + 72,
    // head_of_line,
    // head_of_line + 1,
    // head_of_line + 2,
    // head_of_line + 3,
    // head_of_line + 4,
    // head_of_line + 5,
    // head_of_line + 6,
    // head_of_line + 7,
    // head_of_line + 8,
    // head_of_block,
    // head_of_block + 1,
    // head_of_block + 2,
    // head_of_block + 9,
    // head_of_block + 10,
    // head_of_block + 11,
    // head_of_block + 18,
    // head_of_block + 19,
    // head_of_block + 20]
    //
    ADJACENT_VALUES[index as usize]
}

fn build_possible_values_grid(mut grid: &mut Grid) -> Option<()> {
    build_possible_values_grid_start(grid, 81)
}

fn build_possible_values_grid_start(mut grid: &mut Grid, end: usize) -> Option<()> {

    let mut last_value_changed = 0;

    for index in 0..end {
        let is_possibility = !grid[index].is_value();
        if is_possibility {
            let possible_value = get_possible_values(*grid, index as u8);
            match possible_value {
                Some(CellValue::Possibilities(_)) => {
                    grid[index] = possible_value.unwrap();
                }
                Some(CellValue::Value(new_value)) => {
                    grid[index] = CellValue::Value(new_value);
                    if last_value_changed < index {
                        last_value_changed = index;
                    }
                }
                None => {
                    return None;
                }
            }
        }
    }
    if last_value_changed != 0 {
        // a value was filed before the first value
        // we need to update the previously computed possibilities
        return build_possible_values_grid_start(grid, last_value_changed);
    }
    Some(())
}

fn get_possible_values(grid: Grid, index: u8) -> Option<CellValue> {

    let mut poss = grid[index as usize].clone();

    if let CellValue::Possibilities(ref mut values_again) = poss {

        let values = get_adjacent_cells(index);
        let mut n_poss = values_again.iter().fold(0, |sum, &i| if i { sum + 1 } else { sum });

        for &val in &values {
            if let CellValue::Value(num) = grid[val as usize] {
                if values_again[num as usize - 1] {
                    values_again[num as usize - 1] = false;
                    n_poss -= 1;
                    if n_poss == 0 {
                        // error case: there is no possible values here
                        return None;
                    }
                }
            }
        }

        // There is only one option left
        if n_poss == 1 {
            for (idx, _) in values_again.iter().enumerate().filter(|v| *v.1) {
                let new_cell_value = idx as u8 + 1;
                return Some(CellValue::Value(new_cell_value));
            }
        }

    }

    Some(poss)
}

fn solve_grid(mut grid: Grid) -> Option<Grid> {

    let values_found = build_possible_values_grid(&mut grid);
    if values_found.is_none() {
        return None;
    }

    // start by the number with the lowest possible values already in the grid when guessing
    let res = grid.iter().enumerate().min_by_key(|val| val.1.get_nb_possibility());

    match res {
        Some((index, &cell)) => {
            match cell {
                CellValue::Value(_) => {}
                CellValue::Possibilities(poss) => {
                    let mut new_g = clone_grid(grid);
                    for (idx, &val) in poss.iter().enumerate() {
                        if val {
                            // guess a possible value
                            new_g[index as usize] = CellValue::Value(idx as u8 + 1);
                            if check_grid_at(new_g, index as u8) {
                                if let Some(gx) = solve_grid(new_g) {
                                    return Some(gx);
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
            if !is_grid_complete_full(new_grid) {
                println!("Grid is not correct!");
            }
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

#[cfg(test)]
#[bench]
fn benchmark2(b: &mut Bencher) {

    let grid_string = r#"
 _ _ _   _ _ _   _ _ _
 _ _ _   _ _ 3   _ 8 5
 _ _ 1   _ 2 _   _ _ _

 _ _ _   5 _ 7   _ _ _
 _ _ 4   _ _ _   1 _ _
 _ 9 _   _ _ _   _ _ _

 5 _ _   _ _ _   _ 7 3
 _ _ 2   _ 1 _   _ _ _
 _ _ _   _ 4 _   _ _ 9"#;

    let grid: Grid = parse_grid(grid_string);

    b.iter(|| solve_grid(grid));
}
