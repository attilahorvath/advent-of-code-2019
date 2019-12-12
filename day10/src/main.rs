use std::fs;
use std::io;

use day10::*;

fn main() -> Result<(), io::Error> {
    let input = fs::read_to_string("input.txt")?;
    let map = input.parse::<Map>().unwrap();

    let best_location = map.best_location();

    println!("Maximum asteroids detected: {}", best_location.1);

    let asteroid = map.vaporize(best_location.0).nth(199).unwrap();

    println!("200th asteroid to be vaporized: {}", asteroid);

    Ok(())
}
