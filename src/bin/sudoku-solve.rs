// © Copyright 2021 Sudoku Rustic’s Authors
// Subject to the MIT License. See file LICENSE for details.

use std::env;
use std::error::Error;
use std::io::{self, Read};

pub fn main() -> Result<(), Box<dyn Error>> {
    let mut maximum_solutions = 1;
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        maximum_solutions = args[1].parse()?;
    }
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    let grid = sudoku_rustic::parse(&buffer)?;
    let solutions = sudoku_rustic::solutions(&grid)?;
    for (i, solution) in solutions.enumerate().take(maximum_solutions) {
        if i > 0 {
            println!("\n == Solution {} ==", i + 1);
        }
        let formatted = sudoku_rustic::format(solution);
        print!("{}", formatted);
    }
    Ok(())
}

