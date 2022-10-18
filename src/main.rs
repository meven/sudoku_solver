extern crate num_cpus;
use std::io::{self, Write};
use std::sync::RwLock;

use std::path::PathBuf;

extern crate clap;
use clap::Parser;

#[macro_use]
extern crate error_chain;

extern crate rayon;

use rayon::prelude::*;

use std::fs::File;
use std::io::{BufRead, BufReader};

use std::time::Instant;

mod constants;
use constants::ADJACENT_CELLS;
use constants::ADJACENT_VALUES;

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

    let mut output = String::new();

    for &x in g.iter() {
        cnt += 1;

        match x {
            CellValue::Value(i) => output.push_str(&(i + 1).to_string()),
            CellValue::Possibilities(p) => {
                if with_possibilities {
                    output.push_str("(");
                    for (idx, &val) in p.iter().enumerate() {
                        if val {
                            output.push_str(&(idx + 1).to_string());
                        }
                    }
                    output.push_str(")");
                } else {
                    output.push_str("_");
                }
            }
        }

        if cnt == 9 {
            line += 1;
            output.push_str("\n");
            cnt = 0;
            if line == 3 {
                line = 0;
                output.push_str("\n");
            }
        } else if cnt % 3 == 0 {
            output.push_str("   ");
        } else {
            output.push_str(" ");
        }
    }
    print!("{}", output);
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
                }
                [true, false, false, false, false, false, false, false, false] => {
                    if !set_cell_value_at(grid, val, 0) {
                        return false;
                    }
                }
                [false, true, false, false, false, false, false, false, false] => {
                    if !set_cell_value_at(grid, val, 1) {
                        return false;
                    }
                }
                [false, false, true, false, false, false, false, false, false] => {
                    if !set_cell_value_at(grid, val, 2) {
                        return false;
                    }
                }
                [false, false, false, true, false, false, false, false, false] => {
                    if !set_cell_value_at(grid, val, 3) {
                        return false;
                    }
                }
                [false, false, false, false, true, false, false, false, false] => {
                    if !set_cell_value_at(grid, val, 4) {
                        return false;
                    }
                }
                [false, false, false, false, false, true, false, false, false] => {
                    if !set_cell_value_at(grid, val, 5) {
                        return false;
                    }
                }
                [false, false, false, false, false, false, true, false, false] => {
                    if !set_cell_value_at(grid, val, 6) {
                        return false;
                    }
                }
                [false, false, false, false, false, false, false, true, false] => {
                    if !set_cell_value_at(grid, val, 7) {
                        return false;
                    }
                }
                [false, false, false, false, false, false, false, false, true] => {
                    if !set_cell_value_at(grid, val, 8) {
                        return false;
                    }
                }
                _ => {}
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
    let res = grid
        .iter()
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
            let _ = writeln!(
                handle,
                "Grid complete ! in {} us",
                (1_000_000 * duration.as_secs() + u64::from(duration.subsec_nanos())) / (1_000)
            );
            print_grid(grid);
            print_grid(new_grid);
            if !is_grid_complete_full(new_grid) {
                println!("Grid is not correct!");
            }
        }
        None => {
            let _ = writeln!(
                handle,
                "Couldn't solve the sudoku :( in {} ms",
                (1_000_000 * duration.as_secs() + u64::from(duration.subsec_nanos())) / (1_000)
            );
            print_grid(grid);
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Opt {
    /// Files to process
    #[arg(name = "FILE")]
    file: PathBuf,
}

fn run() -> Result<()> {
    let opt = Opt::parse();

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
