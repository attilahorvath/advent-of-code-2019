use std::error::Error;
use std::fs;

use day12::*;

fn main() -> Result<(), Box<dyn Error>> {
    let positions = fs::read_to_string("input.txt")?;

    let moons = positions
        .lines()
        .map(|position| Ok(position.parse::<Moon>()?))
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

    let mut system = System::new(&moons);

    system.steps(1000);

    println!("Total energy: {}", system.total_energy());

    Ok(())
}
