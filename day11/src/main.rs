use std::error::Error;
use std::fs;

use day11::*;

fn main() -> Result<(), Box<dyn Error>> {
    let program = fs::read_to_string("input.txt")?;
    let mut test_hull = Hull::new();
    let mut robot = Robot::new();

    let panels_painted = robot.run(program.trim(), &mut test_hull, Color::Black)?;

    println!("Panels painted: {}", panels_painted);

    let mut hull = Hull::new();

    robot.run(program.trim(), &mut hull, Color::White)?;

    println!("{}", hull);

    Ok(())
}
