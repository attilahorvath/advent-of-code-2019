use std::error::Error;
use std::fs;

use day15::*;

fn main() -> Result<(), Box<dyn Error>> {
    let program = fs::read_to_string("input.txt")?;
    let mut remote_control = RemoteControl::new(program.trim())?;

    println!("Target depth: {}", remote_control.find_target());
    println!("Max depth: {}", remote_control.find_max_depth());

    Ok(())
}
