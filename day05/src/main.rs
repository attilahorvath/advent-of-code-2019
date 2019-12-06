use std::error::Error;
use std::fs;

use day05::*;

fn main() -> Result<(), Box<dyn Error>> {
    let program = fs::read_to_string("input.txt")?;

    println!("Output for input 1:");

    let mut computer = Computer::new(program.trim())?;
    computer.run(1);

    println!("Output for input 5:");

    computer = Computer::new(program.trim())?;
    computer.run(5);

    Ok(())
}
