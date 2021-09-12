use std::error::Error;
use std::io::{self, Read};

pub fn main() -> Result<(), Box<dyn Error>> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    let grid = sudokur::parse(&buffer)?;
    let formatted = sudokur::format(grid);
    print!("{}", formatted);
    Ok(())
}
