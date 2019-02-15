
# Rust sudoku solver

Yet another sudoku solver, it was made to learn rust and optimize both algorithm and code.
It is inspired by Peter Norvig research at http://norvig.com/sudoku.html
I have reused its hardest sudoku files as test reference.

It uses rayon to parallelize the execution as much as possible.

## Usage
top95.txt contains 95 hard to solve sudokus.

    cargo run --release top95.txt

Or

    cargo build --release
    ./target/release/sudoku_solver top95.txt

## Performance

On my setup, a Rizen 5 1600 with 16Gb of RAM, it results in

    real    0m0,080s
    user    0m0,849s
    sys     0m0,012s

It is able to solve any sudoku in under 100 ms, most of them much faster.

## Other implementation

https://emerentius.github.io/sudoku_web/  uses a better algorithm and can run in the browser using webassembly.

https://github.com/gnuvince/sudoku-rs
Quite a bit slower, but using rust idioms.
