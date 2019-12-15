use std::error::Error;
use std::fs;

use intcode::Computer;

fn main() -> Result<(), Box<dyn Error>> {
    let program = fs::read_to_string("input.txt")?;
    let mut computer = Computer::new(program.trim())?;
    let (sender, receiver) = computer.get_io();

    println!("Output for input 1:");

    sender.send(1)?;
    computer.run();

    for message in receiver.iter() {
        println!("{}", message);

        if message != 0 {
            break;
        }
    }

    println!("Output for input 5:");

    sender.send(5)?;
    computer.run();

    println!("{}", receiver.recv()?);

    Ok(())
}
