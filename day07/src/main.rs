use std::error::Error;
use std::fs;

use day07::*;

fn main() -> Result<(), Box<dyn Error>> {
    let program = fs::read_to_string("input.txt")?;

    println!(
        "Highest signal: {}",
        highest_signal(program.trim(), &mut [0, 1, 2, 3, 4], false)
    );

    println!(
        "Highest signal with feedback: {}",
        highest_signal(program.trim(), &mut [5, 6, 7, 8, 9], true)
    );

    Ok(())
}
