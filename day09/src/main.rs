use std::error::Error;
use std::fs;

use day09::*;

fn main() -> Result<(), Box<dyn Error>> {
    let program = fs::read_to_string("input.txt")?;
    let mut computer = Computer::new(program.trim())?;
    let (sender, receiver) = computer.get_io();

    sender.send(1)?;
    computer.run();

    println!("Keycode: {}", receiver.recv()?);

    sender.send(2)?;
    computer.run();

    println!("Coordinates: {}", receiver.recv()?);

    Ok(())
}
