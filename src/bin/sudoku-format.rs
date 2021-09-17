// © Copyright 2021 Sudoku Rustic’s Authors
// Subject to the MIT License. See file LICENSE for details.

use std::error::Error;
use std::io::{self, Read};

pub fn main() -> Result<(), Box<dyn Error>> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    let grid = sudoku_rustic::parse(&buffer)?;
    let formatted = sudoku_rustic::format(grid);
    print!("{}", formatted);
    Ok(())
}
