use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use day06::*;

fn main() -> Result<(), Box<dyn Error>> {
    let file = BufReader::new(File::open("input.txt")?);

    let entries = file
        .lines()
        .map(|line| Ok(line?.parse::<Entry>()?))
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

    let map = Map::new(entries);

    println!("Total orbits: {}", map.total_orbits());
    println!("Transfers needed: {}", map.transfers_needed());

    Ok(())
}
