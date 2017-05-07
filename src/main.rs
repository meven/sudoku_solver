
extern crate num_cpus;

use std::collections::HashMap;
use std::thread;
use std::time::Instant;

type Possibility = Vec<u8>;
type Grid = [Option<u8>; 81];

fn print_grid(g: Grid) {

    let mut cnt = 0;
    let mut line = 0;

    for &x in g.iter() {

        cnt = cnt + 1;

        match x {
            Some(i) => print!("{}", i),
            None => print!("_"),
        }

        if cnt == 9 {
            line = line + 1;
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
    for &v in val.iter() {
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
    return true;
}

fn get_line(index: usize) -> usize {
    return (index / 9) * 9;
}

fn get_column(index: usize) -> usize {
    return index % 9;
}

fn get_head_of_block(index: usize) -> usize {
    return index - (index % 3) - (index / 9 % 3) * 9;
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

    return true;
}

fn is_grid_complete(g: Grid) -> bool {
    g.iter().all(|&x| x != None)
}

fn clone_grid(g: Grid) -> Grid {
    let mut new_g: Grid = [None; 81];
    for x in 0..g.len() {
        new_g[x] = g[x];
    }
    return new_g;
}

fn build_possible_values_grid(grid: Grid) -> HashMap<usize, Possibility> {
    let mut possible_grid = HashMap::new();

    for index in 0..grid.len() {
        if grid[index] == None {
            let possible_values = get_possible_values(grid, index);
            let len = possible_values.len();
            possible_grid.insert(index, possible_values);
            if len == 0 {
                return possible_grid;
            }
        }
    }
    return possible_grid;
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

    for &val in values.iter() {
        if let Some(num) = grid[val] {
            if !values_again[num as usize - 1] {
                values_again[num as usize - 1] = true;
                n_found = n_found + 1;
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

    return val_found;
}

fn solve_grid(grid: Grid) -> Option<Grid> {

    let possible_values_grid = build_possible_values_grid(grid);
    let length = possible_values_grid.len();

    if length > 0 {
        let (&index, possible_values) = possible_values_grid.iter()
            .min_by_key(|x| x.1.len())
            .unwrap();

        match possible_values.len() {
            0 => return None,
            1 => {
                // fill in all the cells where there is only one possiblitiy
                let mut new_g = clone_grid(grid);

                for (&index, possible_values) in possible_values_grid.iter() {
                    if possible_values.len() == 1 {
                        new_g[index] = Some(possible_values[0]);
                    }
                }

                return solve_grid(new_g);
            }
            _ => {
                for val in possible_values {

                    let mut new_g = clone_grid(grid);
                    new_g[index] = Some(*val);

                    if check_grid_at(new_g, index) {
                        let gx = solve_grid(new_g);
                        if gx.is_some() {
                            return gx;
                        }
                    }
                }
                return None;
            }
        }

    } else if is_grid_complete(grid) {
        return Some(grid);
    } else {
        return None;
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

    return grid;
}

fn main() {

    let grid_strings = [r#"
            1 _ _   _ _ _   _ _ 3
            _ 4 _   _ _ 9   2 6 _
            _ _ _   7 _ _   _ 5 4

            _ _ _   1 7 _   9 _ _
            _ _ 2   _ _ _   6 _ _
            _ _ 3   _ 9 5   _ _ _

            2 7 _   _ _ 1   _ _ _
            _ 8 9   3 _ _   _ 7 _
            6 _ _   _ _ _   _ _ 2"#,

                        r#"
            _ _ _   _ _ _   _ _ _
            _ _ _   _ _ _   _ _ _
            _ _ _   _ _ _   _ _ _

            _ _ _   _ _ _   _ _ _
            _ _ _   _ _ _   _ _ _
            _ _ _   _ _ _   _ _ _

            _ _ _   _ _ _   _ _ _
            _ _ _   _ _ _   _ _ _
            _ _ _   _ _ _   _ _ _"#,

                        r#"
            1 2 3   _ _ _   _ _ _
            4 5 6   _ _ _   _ _ _
            7 8 9   _ _ _   _ _ _

            _ _ _   1 2 3   _ _ _
            _ _ _   4 5 6   _ _ _
            _ _ _   7 8 9   _ _ _

            _ _ _   _ _ _   1 2 3
            _ _ _   _ _ _   4 5 6
            _ _ _   _ _ _   7 8 9"#,

                        r#"
            _ 3 _   _ _ _   _ _ _
            7 _ _   3 _ _   2 _ 5
            _ _ _   _ _ 2   3 _ 9

            5 _ _   8 1 _   _ _ _
            _ 7 9   _ _ _   5 4 _
            _ _ _   _ 5 4   _ _ 7

            6 _ 3   4 _ _   _ _ _
            9 _ 8   _ _ 1   _ _ 3
            _ _ _   _ _ _   _ 2 _"#,
                        r#"
            _ _ _   _ _ _   _ _ _
            _ 1 _   6 _ _   _ _ 8
            _ _ 5   _ 7 _   _ 1 _

            _ _ 8   _ _ _   _ _ _
            _ _ _   4 1 9   _ _ _
            _ _ _   _ _ _   2 _ _

            _ 5 _   _ 3 _   7 _ _
            4 _ _   _ _ 8   _ 9 _
            _ _ _   _ _ _   _ _ _"#,

                        r#"
            _ 6 _   _ 7 _   2 _ _
            _ 2 5   _ _ _   _ 8 _
            4 _ _   2 _ 1   _ 6 3

            _ _ 7   5 _ _   4 _ _
            _ _ _   _ 4 _   _ _ _
            _ _ 1   _ _ 9   3 _ _

            6 9 _   1 _ 2   _ _ 5
            _ 1 _   _ _ _   6 3 _
            _ _ 3   _ 6 _   _ 2 _"#,

                        r#"
            ___2___63
            3____54_1
            __1__398_
            _______9_
            ___538___
            _3_______
            _263__5__
            5_37____8
            47___1___
                "#,
                        r#"
            _____36_5
            ___2_____
            ____7_14_
            24______7
            6__914__2
            8______14
            _18_5____
            _____6___
            9_38_____
            "#];

    // As many threads as cpus
    let nb_threads: usize = num_cpus::get();

    for i in 0..(grid_strings.len() / nb_threads + 1) {

        let mut children = vec![];

        let low = i * nb_threads;
        let mut high = grid_strings.len() - low;
        if high > nb_threads - 1 {
            high = low + nb_threads;
        }

        for index in low..high {
            children.push(thread::spawn(move || {

                let grid: Grid = parse_grid(grid_strings[index]);

                let now = Instant::now();
                let new_grid = solve_grid(grid);
                let duration = now.elapsed();

                match new_grid {
                    Some(new_grid) => {
                        println!("Grid complete ! in {} ms",
                                 (1000_000_000 * duration.as_secs() +
                                  duration.subsec_nanos() as u64) /
                                 (1000_000));
                        print_grid(grid);
                        print_grid(new_grid);
                    }
                    None => {
                        println!("Couldn't solve the sudoku");
                    }
                }
            }));
        }

        for thread in children {
            thread.join().ok();
        }
    }
}

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
