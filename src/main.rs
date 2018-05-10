#![feature(nll)]
#![feature(test)]
#![feature(slice_patterns)]

extern crate num_cpus;
use std::sync::RwLock;
use std::io::{self, Write};
#[macro_use]
extern crate structopt;

use std::path::PathBuf;
use structopt::StructOpt;


#[macro_use]
extern crate error_chain;

extern crate rayon;

use rayon::prelude::*;

use std::fs::File;
use std::io::{BufRead, BufReader};

use std::time::Instant;

error_chain! {
    foreign_links {
        Io(std::io::Error);
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum CellValue {
    Value(usize),
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

    pub fn get_nb_possibility(&self) -> usize {
        match *self {
            CellValue::Value(_) => 10,
            CellValue::Possibilities(values) => {
                values
                    .iter()
                    .fold(0, |acc, &p| if p { acc + 1 } else { acc })
            }
        }
    }
}


fn is_grid_complete_full(g: Grid) -> bool {
    g.iter()
        .enumerate()
        .all(|x| x.1.is_value() && check_grid_at(g, x.0))
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
            CellValue::Value(i) => print!("{}", i + 1),
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
            println!();
            cnt = 0;
            if line == 3 {
                line = 0;
                println!();
            }
        } else if cnt % 3 == 0 {
            print!("   ");
        } else {
            print!(" ");
        }
    }
}

/*
fn get_line(index: usize) -> usize {
    (index / 9) * 9
}

fn get_column(index: usize) -> usize {
    index % 9
}

fn get_head_of_block(index: usize) -> usize {
    index - (index % 3) - (index / 9 % 3) * 9
}
*/

fn check_grid_at(g: Grid, index: usize) -> bool {
    let adj_cells = ADJACENT_CELLS[index];
    //let line = get_line(index);

    if !check_no_redundant_value(g, adj_cells[0]) {
        return false;
    }
    if !check_no_redundant_value(g, adj_cells[1]) {
        return false;
    }
    if !check_no_redundant_value(g, adj_cells[2]) {
        return false;
    }

    true
}

fn check_no_redundant_value(grid: Grid, val: [usize; 8]) -> bool {
    let mut checked: [bool; 9] = [false; 9];
    for &v in &val {
        if let CellValue::Value(cell_value) = grid[v] {
            if checked[cell_value] {
                return false;
            }
            checked[cell_value] = true;
        }
    }
    true
}

static ADJACENT_CELLS: [[[usize; 8]; 3]; 81] = [
    [
        [1, 2, 3, 4, 5, 6, 7, 8],
        [36, 72, 9, 45, 18, 54, 27, 63],
        [1, 2, 9, 10, 11, 18, 19, 20],
    ],
    [
        [0, 2, 3, 4, 5, 6, 7, 8],
        [64, 37, 73, 10, 46, 19, 55, 28],
        [0, 2, 9, 10, 11, 18, 19, 20],
    ],
    [
        [0, 1, 3, 4, 5, 6, 7, 8],
        [65, 38, 74, 11, 47, 20, 56, 29],
        [0, 1, 9, 10, 11, 18, 19, 20],
    ],
    [
        [0, 1, 2, 4, 5, 6, 7, 8],
        [66, 39, 75, 12, 48, 21, 57, 30],
        [4, 5, 12, 13, 14, 21, 22, 23],
    ],
    [
        [0, 1, 2, 3, 5, 6, 7, 8],
        [67, 40, 76, 13, 49, 22, 58, 31],
        [3, 5, 12, 13, 14, 21, 22, 23],
    ],
    [
        [0, 1, 2, 3, 4, 6, 7, 8],
        [32, 68, 41, 77, 14, 50, 23, 59],
        [3, 4, 12, 13, 14, 21, 22, 23],
    ],
    [
        [0, 1, 2, 3, 4, 5, 7, 8],
        [33, 69, 42, 78, 15, 51, 24, 60],
        [7, 8, 15, 16, 17, 24, 25, 26],
    ],
    [
        [0, 1, 2, 3, 4, 5, 6, 8],
        [34, 70, 43, 79, 16, 52, 25, 61],
        [6, 8, 15, 16, 17, 24, 25, 26],
    ],
    [
        [0, 1, 2, 3, 4, 5, 6, 7],
        [35, 71, 44, 80, 17, 53, 26, 62],
        [6, 7, 15, 16, 17, 24, 25, 26],
    ],
    [
        [10, 11, 12, 13, 14, 15, 16, 17],
        [0, 36, 72, 45, 18, 54, 27, 63],
        [0, 1, 2, 10, 11, 18, 19, 20],
    ],
    [
        [9, 11, 12, 13, 14, 15, 16, 17],
        [64, 1, 37, 73, 46, 19, 55, 28],
        [0, 1, 2, 9, 11, 18, 19, 20],
    ],
    [
        [9, 10, 12, 13, 14, 15, 16, 17],
        [65, 2, 38, 74, 47, 20, 56, 29],
        [0, 1, 2, 9, 10, 18, 19, 20],
    ],
    [
        [9, 10, 11, 13, 14, 15, 16, 17],
        [66, 3, 39, 75, 48, 21, 57, 30],
        [3, 4, 5, 13, 14, 21, 22, 23],
    ],
    [
        [9, 10, 11, 12, 14, 15, 16, 17],
        [67, 4, 40, 76, 49, 22, 58, 31],
        [3, 4, 5, 12, 14, 21, 22, 23],
    ],
    [
        [9, 10, 11, 12, 13, 15, 16, 17],
        [32, 68, 5, 41, 77, 50, 23, 59],
        [3, 4, 5, 12, 13, 21, 22, 23],
    ],
    [
        [9, 10, 11, 12, 13, 14, 16, 17],
        [33, 69, 6, 42, 78, 51, 24, 60],
        [6, 7, 8, 16, 17, 24, 25, 26],
    ],
    [
        [9, 10, 11, 12, 13, 14, 15, 17],
        [34, 70, 7, 43, 79, 52, 25, 61],
        [6, 7, 8, 15, 17, 24, 25, 26],
    ],
    [
        [9, 10, 11, 12, 13, 14, 15, 16],
        [35, 71, 8, 44, 80, 53, 26, 62],
        [6, 7, 8, 15, 16, 24, 25, 26],
    ],
    [
        [19, 20, 21, 22, 23, 24, 25, 26],
        [0, 36, 72, 9, 45, 54, 27, 63],
        [0, 1, 2, 9, 10, 11, 19, 20],
    ],
    [
        [18, 20, 21, 22, 23, 24, 25, 26],
        [64, 1, 37, 73, 10, 46, 55, 28],
        [0, 1, 2, 9, 10, 11, 18, 20],
    ],
    [
        [18, 19, 21, 22, 23, 24, 25, 26],
        [65, 2, 38, 74, 11, 47, 56, 29],
        [0, 1, 2, 9, 10, 11, 18, 19],
    ],
    [
        [18, 19, 20, 22, 23, 24, 25, 26],
        [66, 3, 39, 75, 12, 48, 57, 30],
        [3, 4, 5, 12, 13, 14, 22, 23],
    ],
    [
        [18, 19, 20, 21, 23, 24, 25, 26],
        [67, 4, 40, 76, 13, 49, 58, 31],
        [3, 4, 5, 12, 13, 14, 21, 23],
    ],
    [
        [18, 19, 20, 21, 22, 24, 25, 26],
        [32, 68, 5, 41, 77, 14, 50, 59],
        [3, 4, 5, 12, 13, 14, 21, 22],
    ],
    [
        [18, 19, 20, 21, 22, 23, 25, 26],
        [33, 69, 6, 42, 78, 15, 51, 60],
        [6, 7, 8, 15, 16, 17, 25, 26],
    ],
    [
        [18, 19, 20, 21, 22, 23, 24, 26],
        [34, 70, 7, 43, 79, 16, 52, 61],
        [6, 7, 8, 15, 16, 17, 24, 26],
    ],
    [
        [18, 19, 20, 21, 22, 23, 24, 25],
        [35, 71, 8, 44, 80, 17, 53, 62],
        [6, 7, 8, 15, 16, 17, 24, 25],
    ],
    [
        [32, 33, 34, 35, 28, 29, 30, 31],
        [0, 36, 72, 9, 45, 18, 54, 63],
        [36, 37, 38, 45, 46, 47, 28, 29],
    ],
    [
        [32, 33, 34, 35, 27, 29, 30, 31],
        [64, 1, 37, 73, 10, 46, 19, 55],
        [36, 37, 38, 45, 46, 47, 27, 29],
    ],
    [
        [32, 33, 34, 35, 27, 28, 30, 31],
        [65, 2, 38, 74, 11, 47, 20, 56],
        [36, 37, 38, 45, 46, 47, 27, 28],
    ],
    [
        [32, 33, 34, 35, 27, 28, 29, 31],
        [66, 3, 39, 75, 12, 48, 21, 57],
        [32, 39, 40, 41, 48, 49, 50, 31],
    ],
    [
        [32, 33, 34, 35, 27, 28, 29, 30],
        [67, 4, 40, 76, 13, 49, 22, 58],
        [32, 39, 40, 41, 48, 49, 50, 30],
    ],
    [
        [33, 34, 35, 27, 28, 29, 30, 31],
        [68, 5, 41, 77, 14, 50, 23, 59],
        [39, 40, 41, 48, 49, 50, 30, 31],
    ],
    [
        [32, 34, 35, 27, 28, 29, 30, 31],
        [69, 6, 42, 78, 15, 51, 24, 60],
        [34, 35, 42, 43, 44, 51, 52, 53],
    ],
    [
        [32, 33, 35, 27, 28, 29, 30, 31],
        [70, 7, 43, 79, 16, 52, 25, 61],
        [33, 35, 42, 43, 44, 51, 52, 53],
    ],
    [
        [32, 33, 34, 27, 28, 29, 30, 31],
        [71, 8, 44, 80, 17, 53, 26, 62],
        [33, 34, 42, 43, 44, 51, 52, 53],
    ],
    [
        [37, 38, 39, 40, 41, 42, 43, 44],
        [0, 72, 9, 45, 18, 54, 27, 63],
        [37, 38, 45, 46, 47, 27, 28, 29],
    ],
    [
        [36, 38, 39, 40, 41, 42, 43, 44],
        [64, 1, 73, 10, 46, 19, 55, 28],
        [36, 38, 45, 46, 47, 27, 28, 29],
    ],
    [
        [36, 37, 39, 40, 41, 42, 43, 44],
        [65, 2, 74, 11, 47, 20, 56, 29],
        [36, 37, 45, 46, 47, 27, 28, 29],
    ],
    [
        [36, 37, 38, 40, 41, 42, 43, 44],
        [66, 3, 75, 12, 48, 21, 57, 30],
        [32, 40, 41, 48, 49, 50, 30, 31],
    ],
    [
        [36, 37, 38, 39, 41, 42, 43, 44],
        [67, 4, 76, 13, 49, 22, 58, 31],
        [32, 39, 41, 48, 49, 50, 30, 31],
    ],
    [
        [36, 37, 38, 39, 40, 42, 43, 44],
        [32, 68, 5, 77, 14, 50, 23, 59],
        [32, 39, 40, 48, 49, 50, 30, 31],
    ],
    [
        [36, 37, 38, 39, 40, 41, 43, 44],
        [33, 69, 6, 78, 15, 51, 24, 60],
        [33, 34, 35, 43, 44, 51, 52, 53],
    ],
    [
        [36, 37, 38, 39, 40, 41, 42, 44],
        [34, 70, 7, 79, 16, 52, 25, 61],
        [33, 34, 35, 42, 44, 51, 52, 53],
    ],
    [
        [36, 37, 38, 39, 40, 41, 42, 43],
        [35, 71, 8, 80, 17, 53, 26, 62],
        [33, 34, 35, 42, 43, 51, 52, 53],
    ],
    [
        [46, 47, 48, 49, 50, 51, 52, 53],
        [0, 36, 72, 9, 18, 54, 27, 63],
        [36, 37, 38, 46, 47, 27, 28, 29],
    ],
    [
        [45, 47, 48, 49, 50, 51, 52, 53],
        [64, 1, 37, 73, 10, 19, 55, 28],
        [36, 37, 38, 45, 47, 27, 28, 29],
    ],
    [
        [45, 46, 48, 49, 50, 51, 52, 53],
        [65, 2, 38, 74, 11, 20, 56, 29],
        [36, 37, 38, 45, 46, 27, 28, 29],
    ],
    [
        [45, 46, 47, 49, 50, 51, 52, 53],
        [66, 3, 39, 75, 12, 21, 57, 30],
        [32, 39, 40, 41, 49, 50, 30, 31],
    ],
    [
        [45, 46, 47, 48, 50, 51, 52, 53],
        [67, 4, 40, 76, 13, 22, 58, 31],
        [32, 39, 40, 41, 48, 50, 30, 31],
    ],
    [
        [45, 46, 47, 48, 49, 51, 52, 53],
        [32, 68, 5, 41, 77, 14, 23, 59],
        [32, 39, 40, 41, 48, 49, 30, 31],
    ],
    [
        [45, 46, 47, 48, 49, 50, 52, 53],
        [33, 69, 6, 42, 78, 15, 24, 60],
        [33, 34, 35, 42, 43, 44, 52, 53],
    ],
    [
        [45, 46, 47, 48, 49, 50, 51, 53],
        [34, 70, 7, 43, 79, 16, 25, 61],
        [33, 34, 35, 42, 43, 44, 51, 53],
    ],
    [
        [45, 46, 47, 48, 49, 50, 51, 52],
        [35, 71, 8, 44, 80, 17, 26, 62],
        [33, 34, 35, 42, 43, 44, 51, 52],
    ],
    [
        [55, 56, 57, 58, 59, 60, 61, 62],
        [0, 36, 72, 9, 45, 18, 27, 63],
        [64, 65, 72, 73, 74, 55, 56, 63],
    ],
    [
        [54, 56, 57, 58, 59, 60, 61, 62],
        [64, 1, 37, 73, 10, 46, 19, 28],
        [64, 65, 72, 73, 74, 54, 56, 63],
    ],
    [
        [54, 55, 57, 58, 59, 60, 61, 62],
        [65, 2, 38, 74, 11, 47, 20, 29],
        [64, 65, 72, 73, 74, 54, 55, 63],
    ],
    [
        [54, 55, 56, 58, 59, 60, 61, 62],
        [66, 3, 39, 75, 12, 48, 21, 30],
        [66, 67, 68, 75, 76, 77, 58, 59],
    ],
    [
        [54, 55, 56, 57, 59, 60, 61, 62],
        [67, 4, 40, 76, 13, 49, 22, 31],
        [66, 67, 68, 75, 76, 77, 57, 59],
    ],
    [
        [54, 55, 56, 57, 58, 60, 61, 62],
        [32, 68, 5, 41, 77, 14, 50, 23],
        [66, 67, 68, 75, 76, 77, 57, 58],
    ],
    [
        [54, 55, 56, 57, 58, 59, 61, 62],
        [33, 69, 6, 42, 78, 15, 51, 24],
        [69, 70, 71, 78, 79, 80, 61, 62],
    ],
    [
        [54, 55, 56, 57, 58, 59, 60, 62],
        [34, 70, 7, 43, 79, 16, 52, 25],
        [69, 70, 71, 78, 79, 80, 60, 62],
    ],
    [
        [54, 55, 56, 57, 58, 59, 60, 61],
        [35, 71, 8, 44, 80, 17, 53, 26],
        [69, 70, 71, 78, 79, 80, 60, 61],
    ],
    [
        [64, 65, 66, 67, 68, 69, 70, 71],
        [0, 36, 72, 9, 45, 18, 54, 27],
        [64, 65, 72, 73, 74, 54, 55, 56],
    ],
    [
        [65, 66, 67, 68, 69, 70, 71, 63],
        [1, 37, 73, 10, 46, 19, 55, 28],
        [65, 72, 73, 74, 54, 55, 56, 63],
    ],
    [
        [64, 66, 67, 68, 69, 70, 71, 63],
        [2, 38, 74, 11, 47, 20, 56, 29],
        [64, 72, 73, 74, 54, 55, 56, 63],
    ],
    [
        [64, 65, 67, 68, 69, 70, 71, 63],
        [3, 39, 75, 12, 48, 21, 57, 30],
        [67, 68, 75, 76, 77, 57, 58, 59],
    ],
    [
        [64, 65, 66, 68, 69, 70, 71, 63],
        [4, 40, 76, 13, 49, 22, 58, 31],
        [66, 68, 75, 76, 77, 57, 58, 59],
    ],
    [
        [64, 65, 66, 67, 69, 70, 71, 63],
        [32, 5, 41, 77, 14, 50, 23, 59],
        [66, 67, 75, 76, 77, 57, 58, 59],
    ],
    [
        [64, 65, 66, 67, 68, 70, 71, 63],
        [33, 6, 42, 78, 15, 51, 24, 60],
        [70, 71, 78, 79, 80, 60, 61, 62],
    ],
    [
        [64, 65, 66, 67, 68, 69, 71, 63],
        [34, 7, 43, 79, 16, 52, 25, 61],
        [69, 71, 78, 79, 80, 60, 61, 62],
    ],
    [
        [64, 65, 66, 67, 68, 69, 70, 63],
        [35, 8, 44, 80, 17, 53, 26, 62],
        [69, 70, 78, 79, 80, 60, 61, 62],
    ],
    [
        [73, 74, 75, 76, 77, 78, 79, 80],
        [0, 36, 9, 45, 18, 54, 27, 63],
        [64, 65, 73, 74, 54, 55, 56, 63],
    ],
    [
        [72, 74, 75, 76, 77, 78, 79, 80],
        [64, 1, 37, 10, 46, 19, 55, 28],
        [64, 65, 72, 74, 54, 55, 56, 63],
    ],
    [
        [72, 73, 75, 76, 77, 78, 79, 80],
        [65, 2, 38, 11, 47, 20, 56, 29],
        [64, 65, 72, 73, 54, 55, 56, 63],
    ],
    [
        [72, 73, 74, 76, 77, 78, 79, 80],
        [66, 3, 39, 12, 48, 21, 57, 30],
        [66, 67, 68, 76, 77, 57, 58, 59],
    ],
    [
        [72, 73, 74, 75, 77, 78, 79, 80],
        [67, 4, 40, 13, 49, 22, 58, 31],
        [66, 67, 68, 75, 77, 57, 58, 59],
    ],
    [
        [72, 73, 74, 75, 76, 78, 79, 80],
        [32, 68, 5, 41, 14, 50, 23, 59],
        [66, 67, 68, 75, 76, 57, 58, 59],
    ],
    [
        [72, 73, 74, 75, 76, 77, 79, 80],
        [33, 69, 6, 42, 15, 51, 24, 60],
        [69, 70, 71, 79, 80, 60, 61, 62],
    ],
    [
        [72, 73, 74, 75, 76, 77, 78, 80],
        [34, 70, 7, 43, 16, 52, 25, 61],
        [69, 70, 71, 78, 80, 60, 61, 62],
    ],
    [
        [72, 73, 74, 75, 76, 77, 78, 79],
        [35, 71, 8, 44, 17, 53, 26, 62],
        [69, 70, 71, 78, 79, 60, 61, 62],
    ],
];

static ADJACENT_VALUES: [[usize; 20]; 81] = [
    [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 18, 19, 20, 27, 36, 45, 54, 63, 72
    ],
    [
        0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 18, 19, 20, 28, 37, 46, 55, 64, 73
    ],
    [
        0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 18, 19, 20, 29, 38, 47, 56, 65, 74
    ],
    [
        0, 1, 2, 4, 5, 6, 7, 8, 12, 13, 14, 21, 22, 23, 30, 39, 48, 57, 66, 75
    ],
    [
        0, 1, 2, 3, 5, 6, 7, 8, 12, 13, 14, 21, 22, 23, 31, 40, 49, 58, 67, 76
    ],
    [
        0, 1, 2, 3, 4, 6, 7, 8, 12, 13, 14, 21, 22, 23, 32, 41, 50, 59, 68, 77
    ],
    [
        0, 1, 2, 3, 4, 5, 7, 8, 15, 16, 17, 24, 25, 26, 33, 42, 51, 60, 69, 78
    ],
    [
        0, 1, 2, 3, 4, 5, 6, 8, 15, 16, 17, 24, 25, 26, 34, 43, 52, 61, 70, 79
    ],
    [
        0, 1, 2, 3, 4, 5, 6, 7, 15, 16, 17, 24, 25, 26, 35, 44, 53, 62, 71, 80
    ],
    [
        0, 1, 2, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 27, 36, 45, 54, 63, 72
    ],
    [
        0, 1, 2, 9, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 28, 37, 46, 55, 64, 73
    ],
    [
        0, 1, 2, 9, 10, 12, 13, 14, 15, 16, 17, 18, 19, 20, 29, 38, 47, 56, 65, 74
    ],
    [
        3, 4, 5, 9, 10, 11, 13, 14, 15, 16, 17, 21, 22, 23, 30, 39, 48, 57, 66, 75
    ],
    [
        3, 4, 5, 9, 10, 11, 12, 14, 15, 16, 17, 21, 22, 23, 31, 40, 49, 58, 67, 76
    ],
    [
        3, 4, 5, 9, 10, 11, 12, 13, 15, 16, 17, 21, 22, 23, 32, 41, 50, 59, 68, 77
    ],
    [
        6, 7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 24, 25, 26, 33, 42, 51, 60, 69, 78
    ],
    [
        6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17, 24, 25, 26, 34, 43, 52, 61, 70, 79
    ],
    [
        6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 24, 25, 26, 35, 44, 53, 62, 71, 80
    ],
    [
        0, 1, 2, 9, 10, 11, 19, 20, 21, 22, 23, 24, 25, 26, 27, 36, 45, 54, 63, 72
    ],
    [
        0, 1, 2, 9, 10, 11, 18, 20, 21, 22, 23, 24, 25, 26, 28, 37, 46, 55, 64, 73
    ],
    [
        0, 1, 2, 9, 10, 11, 18, 19, 21, 22, 23, 24, 25, 26, 29, 38, 47, 56, 65, 74
    ],
    [
        3, 4, 5, 12, 13, 14, 18, 19, 20, 22, 23, 24, 25, 26, 30, 39, 48, 57, 66, 75
    ],
    [
        3, 4, 5, 12, 13, 14, 18, 19, 20, 21, 23, 24, 25, 26, 31, 40, 49, 58, 67, 76
    ],
    [
        3, 4, 5, 12, 13, 14, 18, 19, 20, 21, 22, 24, 25, 26, 32, 41, 50, 59, 68, 77
    ],
    [
        6, 7, 8, 15, 16, 17, 18, 19, 20, 21, 22, 23, 25, 26, 33, 42, 51, 60, 69, 78
    ],
    [
        6, 7, 8, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 26, 34, 43, 52, 61, 70, 79
    ],
    [
        6, 7, 8, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 35, 44, 53, 62, 71, 80
    ],
    [
        0, 9, 18, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 45, 46, 47, 54, 63, 72
    ],
    [
        1, 10, 19, 27, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 45, 46, 47, 55, 64, 73
    ],
    [
        2, 11, 20, 27, 28, 30, 31, 32, 33, 34, 35, 36, 37, 38, 45, 46, 47, 56, 65, 74
    ],
    [
        3, 12, 21, 27, 28, 29, 31, 32, 33, 34, 35, 39, 40, 41, 48, 49, 50, 57, 66, 75
    ],
    [
        4, 13, 22, 27, 28, 29, 30, 32, 33, 34, 35, 39, 40, 41, 48, 49, 50, 58, 67, 76
    ],
    [
        5, 14, 23, 27, 28, 29, 30, 31, 33, 34, 35, 39, 40, 41, 48, 49, 50, 59, 68, 77
    ],
    [
        6, 15, 24, 27, 28, 29, 30, 31, 32, 34, 35, 42, 43, 44, 51, 52, 53, 60, 69, 78
    ],
    [
        7, 16, 25, 27, 28, 29, 30, 31, 32, 33, 35, 42, 43, 44, 51, 52, 53, 61, 70, 79
    ],
    [
        8, 17, 26, 27, 28, 29, 30, 31, 32, 33, 34, 42, 43, 44, 51, 52, 53, 62, 71, 80
    ],
    [
        0, 9, 18, 27, 28, 29, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 54, 63, 72
    ],
    [
        1, 10, 19, 27, 28, 29, 36, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 55, 64, 73
    ],
    [
        2, 11, 20, 27, 28, 29, 36, 37, 39, 40, 41, 42, 43, 44, 45, 46, 47, 56, 65, 74
    ],
    [
        3, 12, 21, 30, 31, 32, 36, 37, 38, 40, 41, 42, 43, 44, 48, 49, 50, 57, 66, 75
    ],
    [
        4, 13, 22, 30, 31, 32, 36, 37, 38, 39, 41, 42, 43, 44, 48, 49, 50, 58, 67, 76
    ],
    [
        5, 14, 23, 30, 31, 32, 36, 37, 38, 39, 40, 42, 43, 44, 48, 49, 50, 59, 68, 77
    ],
    [
        6, 15, 24, 33, 34, 35, 36, 37, 38, 39, 40, 41, 43, 44, 51, 52, 53, 60, 69, 78
    ],
    [
        7, 16, 25, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 44, 51, 52, 53, 61, 70, 79
    ],
    [
        8, 17, 26, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 51, 52, 53, 62, 71, 80
    ],
    [
        0, 9, 18, 27, 28, 29, 36, 37, 38, 46, 47, 48, 49, 50, 51, 52, 53, 54, 63, 72
    ],
    [
        1, 10, 19, 27, 28, 29, 36, 37, 38, 45, 47, 48, 49, 50, 51, 52, 53, 55, 64, 73
    ],
    [
        2, 11, 20, 27, 28, 29, 36, 37, 38, 45, 46, 48, 49, 50, 51, 52, 53, 56, 65, 74
    ],
    [
        3, 12, 21, 30, 31, 32, 39, 40, 41, 45, 46, 47, 49, 50, 51, 52, 53, 57, 66, 75
    ],
    [
        4, 13, 22, 30, 31, 32, 39, 40, 41, 45, 46, 47, 48, 50, 51, 52, 53, 58, 67, 76
    ],
    [
        5, 14, 23, 30, 31, 32, 39, 40, 41, 45, 46, 47, 48, 49, 51, 52, 53, 59, 68, 77
    ],
    [
        6, 15, 24, 33, 34, 35, 42, 43, 44, 45, 46, 47, 48, 49, 50, 52, 53, 60, 69, 78
    ],
    [
        7, 16, 25, 33, 34, 35, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 53, 61, 70, 79
    ],
    [
        8, 17, 26, 33, 34, 35, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 62, 71, 80
    ],
    [
        0, 9, 18, 27, 36, 45, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 72, 73, 74
    ],
    [
        1, 10, 19, 28, 37, 46, 54, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 72, 73, 74
    ],
    [
        2, 11, 20, 29, 38, 47, 54, 55, 57, 58, 59, 60, 61, 62, 63, 64, 65, 72, 73, 74
    ],
    [
        3, 12, 21, 30, 39, 48, 54, 55, 56, 58, 59, 60, 61, 62, 66, 67, 68, 75, 76, 77
    ],
    [
        4, 13, 22, 31, 40, 49, 54, 55, 56, 57, 59, 60, 61, 62, 66, 67, 68, 75, 76, 77
    ],
    [
        5, 14, 23, 32, 41, 50, 54, 55, 56, 57, 58, 60, 61, 62, 66, 67, 68, 75, 76, 77
    ],
    [
        6, 15, 24, 33, 42, 51, 54, 55, 56, 57, 58, 59, 61, 62, 69, 70, 71, 78, 79, 80
    ],
    [
        7, 16, 25, 34, 43, 52, 54, 55, 56, 57, 58, 59, 60, 62, 69, 70, 71, 78, 79, 80
    ],
    [
        8, 17, 26, 35, 44, 53, 54, 55, 56, 57, 58, 59, 60, 61, 69, 70, 71, 78, 79, 80
    ],
    [
        0, 9, 18, 27, 36, 45, 54, 55, 56, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74
    ],
    [
        1, 10, 19, 28, 37, 46, 54, 55, 56, 63, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74
    ],
    [
        2, 11, 20, 29, 38, 47, 54, 55, 56, 63, 64, 66, 67, 68, 69, 70, 71, 72, 73, 74
    ],
    [
        3, 12, 21, 30, 39, 48, 57, 58, 59, 63, 64, 65, 67, 68, 69, 70, 71, 75, 76, 77
    ],
    [
        4, 13, 22, 31, 40, 49, 57, 58, 59, 63, 64, 65, 66, 68, 69, 70, 71, 75, 76, 77
    ],
    [
        5, 14, 23, 32, 41, 50, 57, 58, 59, 63, 64, 65, 66, 67, 69, 70, 71, 75, 76, 77
    ],
    [
        6, 15, 24, 33, 42, 51, 60, 61, 62, 63, 64, 65, 66, 67, 68, 70, 71, 78, 79, 80
    ],
    [
        7, 16, 25, 34, 43, 52, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 71, 78, 79, 80
    ],
    [
        8, 17, 26, 35, 44, 53, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 78, 79, 80
    ],
    [
        0, 9, 18, 27, 36, 45, 54, 55, 56, 63, 64, 65, 73, 74, 75, 76, 77, 78, 79, 80
    ],
    [
        1, 10, 19, 28, 37, 46, 54, 55, 56, 63, 64, 65, 72, 74, 75, 76, 77, 78, 79, 80
    ],
    [
        2, 11, 20, 29, 38, 47, 54, 55, 56, 63, 64, 65, 72, 73, 75, 76, 77, 78, 79, 80
    ],
    [
        3, 12, 21, 30, 39, 48, 57, 58, 59, 66, 67, 68, 72, 73, 74, 76, 77, 78, 79, 80
    ],
    [
        4, 13, 22, 31, 40, 49, 57, 58, 59, 66, 67, 68, 72, 73, 74, 75, 77, 78, 79, 80
    ],
    [
        5, 14, 23, 32, 41, 50, 57, 58, 59, 66, 67, 68, 72, 73, 74, 75, 76, 78, 79, 80
    ],
    [
        6, 15, 24, 33, 42, 51, 60, 61, 62, 69, 70, 71, 72, 73, 74, 75, 76, 77, 79, 80
    ],
    [
        7, 16, 25, 34, 43, 52, 60, 61, 62, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 80
    ],
    [
        8, 17, 26, 35, 44, 53, 60, 61, 62, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79
    ],
];

fn get_adjacent_cells(index: usize) -> [usize; 20] {
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
    ADJACENT_VALUES[index]
}

fn build_possible_values_grid(grid: &mut Grid) -> bool {
    for index in 0..81 {
        if !grid[index].is_value() {
            let possible_value = get_cell_value(*grid, index);
            if let CellValue::Possibilities(poss) = possible_value {
                match possible_value.get_nb_possibility() {
                    0 => {
                        return false;
                    }
                    1 => {
                        if !set_cell_value_at(grid, index, get_last_value_possible(poss)) {
                            return false;
                        }
                    }
                    _ => {
                        grid[index] = possible_value;
                    }
                }
            }
        }
    }
    true
}

fn get_cell_value(grid: Grid, index: usize) -> CellValue {
    let mut possible_values = [true; 9];

    for &val in &get_adjacent_cells(index) {
        if let CellValue::Value(num) = grid[val] {
            if possible_values[num] {
                possible_values[num] = false;
            }
        }
    }

    CellValue::Possibilities(possible_values)
}

fn get_last_value_possible(possible_values: [bool; 9]) -> usize {
    // There is only one option left
    match possible_values.iter().enumerate().find(|v| *v.1) {
        // error case, should never happen
        None => 11,
        Some((idx, _)) => idx,
    }
}

fn fill_one_possibility_cells(grid: &mut Grid, values: [usize; 20]) -> bool {
    for &val in &values {
        if let CellValue::Possibilities(possible_values) = grid[val] {
            
            match possible_values {
                [false, false, false, false, false, false, false, false, false] => {
                    return false;
                },
                [true, false, false, false, false, false, false, false, false] => {
                    if !set_cell_value_at(grid, val, 0) {
                        return false;
                    }
                },
                [false, true, false, false, false, false, false, false, false] => {
                    if !set_cell_value_at(grid, val, 1) {
                        return false;
                    }
                },
                [false, false, true, false, false, false, false, false, false] => {
                    if !set_cell_value_at(grid, val, 2) {
                        return false;
                    }
                },
                [false, false, false, true, false, false, false, false, false] => {
                    if !set_cell_value_at(grid, val, 3) {
                        return false;
                    }
                },
                [false, false, false, false, true, false, false, false, false] => {
                    if !set_cell_value_at(grid, val, 4) {
                        return false;
                    }
                },
                [false, false, false, false, false, true, false, false, false] => {
                    if !set_cell_value_at(grid, val, 5) {
                        return false;
                    }
                },
                [false, false, false, false, false, false, true, false, false] => {
                    if !set_cell_value_at(grid, val, 6) {
                        return false;
                    }
                },
                [false, false, false, false, false, false, false, true, false] => {
                    if !set_cell_value_at(grid, val, 7) {
                        return false;
                    }
                },
                [false, false, false, false, false, false, false, false, true] => {
                    if !set_cell_value_at(grid, val, 8) {
                        return false;
                    }
                }
                _ => {
                }
            }
            
            /*
            match grid[val].get_nb_possibility() {
                0 => {
                    return false;
                }
                1 => {
                    if !set_cell_value_at(grid, val, get_last_value_possible(possible_values)) {
                        return false;
                    }
                }
                _ => {}
            }
            */
        }
    }
    true
}

fn set_cell_value_at(grid: &mut Grid, index: usize, cell_value: usize) -> bool {
    grid[index] = CellValue::Value(cell_value);

    let adjs = get_adjacent_cells(index);

    adjs.iter().for_each(|val| {
        if let CellValue::Possibilities(ref mut possible_values) = grid[*val] {
            if possible_values[cell_value] {
                possible_values[cell_value] = false;
            }
        }
    });

    fill_one_possibility_cells(grid, adjs)
}

fn solve_grid(mut grid: Grid) -> Option<Grid> {
    if !build_possible_values_grid(&mut grid) {
        return None;
    }

    let g: Option<Grid> = None;
    let counter = RwLock::new(g);

    solve_grid_recurse(grid, &counter)
}

fn solve_grid_recurse(grid: Grid, counter: &RwLock<Option<Grid>>) -> Option<Grid> {
    // start by the number with the lowest possible values already in the grid when guessing
    let res = grid.iter()
        .enumerate()
        .filter(|t: &(usize, &CellValue)| !t.1.is_value())
        .min_by_key(|val| val.1.get_nb_possibility());

    if let Some((index, &CellValue::Possibilities(poss))) = res {
        poss.par_iter()
            .enumerate()
            .filter(|t: &(usize, &bool)| *t.1)
            .for_each(|t: (usize, &bool)| {
                let (cell_value, _) = t;
                if counter.read().unwrap().is_none() {
                    let mut new_g = grid;
                    if set_cell_value_at(&mut new_g, index, cell_value)
                        && counter.read().unwrap().is_none()
                    {
                        if let Some(gx) = solve_grid_recurse(new_g, counter) {
                            let mut gres = counter.write().unwrap();
                            *gres = Some(gx);
                        }
                    }
                }
            });

        return *counter.read().unwrap();
    }

    Some(grid)
}

fn parse_grid(grid_string: &str) -> Grid {
    let mut grid = [CellValue::Possibilities([true; 9]); 81];

    let mut i = 0;
    for splitted in grid_string.split_whitespace() {
        for s in splitted.split("") {
            match s {
                "" => {}
                "_" | "." => {
                    i += 1;
                }
                val => {
                    grid[i] = CellValue::Value(val.parse::<usize>().unwrap() - 1);
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

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    match new_grid {
        Some(new_grid) => {
            let _ = write!(
                handle,
                "Grid complete ! in {} us\n",
                (1_000_000 * duration.as_secs() + u64::from(duration.subsec_nanos())) / (1_000)
            );
            print_grid(grid);
            print_grid(new_grid);
            if !is_grid_complete_full(new_grid) {
                println!("Grid is not correct!");
            }
        }
        None => {
            let _ = write!(
                handle,
                "Couldn't solve the sudoku :( in {} ms\n",
                (1_000_000 * duration.as_secs() + u64::from(duration.subsec_nanos())) / (1_000)
            );
            print_grid(grid);
        }
    }
}

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "sudoku_solver")]
struct Opt {
    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    file: PathBuf,
}

fn run() -> Result<()> {
    let opt = Opt::from_args();
    
    let mut grid_strings = vec![];
    /*
    // multi line grid format
    {
        let input = File::open(opt.file)?;
        let buffered = BufReader::new(input);

        let mut line_buffer = vec![];
        for line in buffered.lines() {
            let line_content = line?;
            if !line_content.is_empty() {
                line_buffer.push(line_content);
            }
        }

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
    } 
    */

    // single line
    {
        //let path = "../sudoku-rs/problems.txt";
        // let path = "../sudoku-rs/very_hard.txt";
        //let path = "./hardest.txt";
        //let path = "./locat.txt";
        // let path = "top95.txt";
        let input = File::open(opt.file)?;
        let buffered = BufReader::new(input);

        for line in buffered.lines() {
            let line_content = line?;
            if !line_content.is_empty() {
                grid_strings.push(line_content);
            }
        }
    }

    grid_strings
        .par_iter()
        .for_each(|grid_string_local| treat_grid(grid_string_local));

    Ok(())
}

quick_main!(run);
