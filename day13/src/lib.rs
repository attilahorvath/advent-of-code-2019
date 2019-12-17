use std::cmp::Ordering;
use std::collections::HashMap;
use std::thread;

use intcode::{Computer, Io, ProgramParseError, ValueType};

#[derive(Clone, Copy, PartialEq)]
enum Tile {
    Empty = 0,
    Wall = 1,
    Block = 2,
    Paddle = 3,
    Ball = 4,
    Score = -1,
}

impl From<ValueType> for Tile {
    fn from(value: ValueType) -> Self {
        match value {
            x if x == Tile::Empty as ValueType => Tile::Empty,
            x if x == Tile::Wall as ValueType => Tile::Wall,
            x if x == Tile::Block as ValueType => Tile::Block,
            x if x == Tile::Paddle as ValueType => Tile::Paddle,
            x if x == Tile::Ball as ValueType => Tile::Ball,
            _ => Tile::Empty,
        }
    }
}

struct TileBuilder {
    x: Option<ValueType>,
    y: Option<ValueType>,
    tile: Option<Tile>,
}

impl TileBuilder {
    fn new() -> Self {
        Self {
            x: None,
            y: None,
            tile: None,
        }
    }

    fn process(&mut self, value: ValueType) -> Option<(ValueType, ValueType, Tile)> {
        if self.x.is_none() {
            self.x = Some(value);

            None
        } else if self.y.is_none() {
            self.y = Some(value);

            None
        } else {
            self.tile = Some(value.into());

            let result = if self.x == Some(-1) && self.y == Some(0) {
                Some((value, 0, Tile::Score))
            } else {
                Some((self.x.unwrap(), self.y.unwrap(), self.tile.unwrap()))
            };

            self.x = None;
            self.y = None;
            self.tile = None;

            result
        }
    }
}

pub fn test_game(program: &str) -> Result<usize, ProgramParseError> {
    let mut computer = Computer::new(program)?;

    let mut tile_builder = TileBuilder::new();
    let mut tiles = HashMap::new();

    let (_, receiver) = computer.get_io();

    let thread = thread::spawn(move || {
        computer.run();
    });

    for value in receiver.iter() {
        if let Some(tile) = tile_builder.process(value) {
            tiles.insert((tile.0, tile.1), tile.2);
        }
    }

    thread.join().unwrap();

    Ok(tiles.values().filter(|&&tile| tile == Tile::Block).count())
}

struct Arcade {
    tile_builder: TileBuilder,
    ball_position: (ValueType, ValueType),
    paddle_position: (ValueType, ValueType),
    score: ValueType,
}

impl Io for Arcade {
    fn send(&mut self, value: ValueType) {
        if let Some(tile) = self.tile_builder.process(value) {
            match tile.2 {
                Tile::Ball => self.ball_position = (tile.0, tile.1),
                Tile::Paddle => self.paddle_position = (tile.0, tile.1),
                Tile::Score => {
                    self.score = tile.0;
                    println!("Score: {}", self.score);
                }
                _ => (),
            }
        }
    }

    fn receive(&mut self) -> ValueType {
        match self.paddle_position.0.cmp(&self.ball_position.0) {
            Ordering::Less => 1,
            Ordering::Equal => 0,
            Ordering::Greater => -1,
        }
    }
}

impl Arcade {
    fn new() -> Self {
        Self {
            tile_builder: TileBuilder::new(),
            ball_position: (0, 0),
            paddle_position: (0, 0),
            score: 0,
        }
    }
}

pub fn run_game(program: &str) -> Result<(), ProgramParseError> {
    let mut computer = Computer::new(program)?;
    let arcade = Arcade::new();

    computer.attach_io(Box::new(arcade));

    computer.run_with_values(0, &[2]);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_tiles() {
        assert_eq!(Ok(2), test_game(&"104,1,104,2,104,3,104,6,104,5,104,4,99"));
    }
}
