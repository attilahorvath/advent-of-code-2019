use std::fs::File;
use std::io::{BufRead, BufReader};

use day03::*;

fn main() -> Result<(), std::io::Error> {
    let file = BufReader::new(File::open("input.txt")?);

    let wires = file
        .lines()
        .map(|line| {
            line.expect("error reading file")
                .parse::<Wire>()
                .expect("invalid wire definition")
        })
        .collect::<Vec<_>>();

    let closest_intersection = wires[0]
        .closest_intersection_with(&wires[1])
        .expect("no intersection found");

    let fewest_steps = wires[0]
        .fewest_steps_with(&wires[1])
        .expect("no intersection found");

    println!("Closest intersection: {}", closest_intersection);
    println!("Fewest steps: {}", fewest_steps);

    Ok(())
}
