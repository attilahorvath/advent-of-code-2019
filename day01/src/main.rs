use std::fs::File;
use std::io::{BufRead, BufReader};

use day01::*;

fn main() -> Result<(), std::io::Error> {
    let file = BufReader::new(File::open("input.txt")?);

    let masses = file
        .lines()
        .map(|line| {
            line.expect("error reading file")
                .parse::<i32>()
                .expect("invalid mass")
        })
        .collect::<Vec<_>>();

    let fuel_for_modules = masses
        .iter()
        .map(|&mass| fuel_for_module(mass))
        .sum::<i32>();

    let total_fuel_for_modules = masses
        .iter()
        .map(|&mass| total_fuel_for_module(mass))
        .sum::<i32>();

    println!("Raw fuel required: {}", fuel_for_modules);
    println!("Total fuel required: {}", total_fuel_for_modules);

    Ok(())
}
