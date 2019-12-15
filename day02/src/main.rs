use std::error::Error;
use std::fs;

use intcode::{Computer, ValueType};

const ORIGINAL_OUTPUT: ValueType = 19_690_720;

fn main() -> Result<(), Box<dyn Error>> {
    let program = fs::read_to_string("input.txt")?;
    let mut computer = Computer::new(program.trim())?;

    computer.run_with_values(1, &[12, 2]);

    println!("Output: {}", computer.dma(0));

    'outer: for noun in 0..=99 {
        for verb in 0..=99 {
            computer.run_with_values(1, &[noun, verb]);

            if *computer.dma(0) == ORIGINAL_OUTPUT {
                println!("Original inputs: {}", noun * 100 + verb);
                break 'outer;
            }
        }
    }

    Ok(())
}
