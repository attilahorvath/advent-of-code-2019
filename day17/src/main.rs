use std::error::Error;
use std::fs;

use day17::*;

fn main() -> Result<(), Box<dyn Error>> {
    let program = fs::read_to_string("input.txt")?;
    let alignment = calculate_alignment(program.trim())?;

    println!("Sum of alignment parameters: {}", alignment);

    Ok(())
}
