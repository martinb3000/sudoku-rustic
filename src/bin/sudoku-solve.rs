use std::error::Error;
use std::io::{self, Read};

pub fn main() -> Result<(), Box<dyn Error>> {
    let maximum_solutions = 1; // TODO: Parse argument.
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    let grid = sudokur::parse(&buffer)?;
    let solutions = sudokur::solutions(&grid)?;
    for (i, solution) in solutions.enumerate().take(maximum_solutions) {
        if i > 0 {
            println!("\n == Solution {} ==\n", i + 1);
        }
        let formatted = sudokur::format(solution);
        print!("{}", formatted);
    }
    Ok(())
}
