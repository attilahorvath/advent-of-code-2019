use std::error::Error;
use std::fs;

use day08::*;

fn main() -> Result<(), Box<dyn Error>> {
    let data = fs::read_to_string("input.txt")?;
    let image = Image::parse(&data, 25, 6);

    println!("Checksum: {}", image.checksum());
    println!("{}", image);

    Ok(())
}
