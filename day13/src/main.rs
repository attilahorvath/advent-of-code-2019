use std::error::Error;
use std::fs;

use day13::*;

fn main() -> Result<(), Box<dyn Error>> {
    let program = fs::read_to_string("input.txt")?;

    println!("Tiles: {}", test_game(program.trim()).unwrap_or(0));

    run_game(program.trim())?;

    Ok(())
}
