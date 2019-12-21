use std::error::Error;
use std::fs;

use day19::*;

fn main() -> Result<(), Box<dyn Error>> {
    let program = fs::read_to_string("input.txt")?;

    let mut beam = Beam::new(program.trim())?;

    println!("Area affected: {}", beam.area_affected(50));

    let square = beam.find_square(100);

    println!("Square corner at: {}", square.0 * 10_000 + square.1);

    Ok(())
}
