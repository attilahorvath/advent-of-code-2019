use std::error::Error;
use std::fs;

use day14::*;

fn main() -> Result<(), Box<dyn Error>> {
    let reactions = fs::read_to_string("input.txt")?
        .lines()
        .map(str::parse::<Reaction>)
        .collect::<Result<Vec<_>, _>>()?;

    let reactor = Reactor::new(reactions);

    println!("Ore cost for 1 FUEL: {}", reactor.fuel_cost());
    println!("Maximum amount of FUEL: {}", reactor.max_fuel());

    Ok(())
}
