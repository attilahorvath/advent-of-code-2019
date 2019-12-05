use std::fs;

use day02::*;

fn main() -> Result<(), std::io::Error> {
    let file = fs::read_to_string("input.txt")?;

    let memory = file
        .trim()
        .split(",")
        .map(|elem| elem.parse::<i32>().expect("invalid integer"))
        .collect::<Vec<_>>();

    let mut computer = Computer::new(&memory).with_inputs(12, 2);

    computer.run();

    println!("Output: {}", computer.output());

    'outer: for noun in 0..=99 {
        for verb in 0..=99 {
            let mut test_computer = Computer::new(&memory).with_inputs(noun, verb);

            test_computer.run();

            if test_computer.output() == 19690720 {
                println!("Original inputs: {}", noun * 100 + verb);
                break 'outer;
            }
        }
    }

    Ok(())
}
