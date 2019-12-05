use std::fs;

use day02::*;

fn main() -> Result<(), std::io::Error> {
    let file = fs::read_to_string("input.txt")?;

    let mut memory = file
        .trim()
        .split(",")
        .map(|elem| elem.parse::<i32>().expect("invalid integer"))
        .collect::<Vec<_>>();

    memory[1] = 12;
    memory[2] = 2;

    run(&mut memory);

    println!("Result: {}", memory[0]);

    Ok(())
}
