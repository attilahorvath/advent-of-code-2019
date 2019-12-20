use std::thread;

use intcode::{Computer, ProgramParseError};

#[derive(Clone, Copy, PartialEq)]
enum Tile {
    Space = '.' as isize,
    Scaffold = '#' as isize,
    Unknown,
}

impl From<char> for Tile {
    fn from(c: char) -> Tile {
        match c as u8 {
            x if x == Tile::Space as u8 => Tile::Space,
            x if x == Tile::Scaffold as u8 => Tile::Scaffold,
            _ => Tile::Unknown,
        }
    }
}

struct Map {
    tiles: Vec<Vec<Tile>>,
}

impl Map {
    fn new() -> Self {
        Self {
            tiles: vec![vec![]],
        }
    }

    fn append(&mut self, c: char) {
        let height = self.tiles.len();

        match c {
            '\n' => self.tiles.push(vec![]),
            _ => self.tiles[height - 1].push(Tile::from(c)),
        }
    }

    fn get(&self, x: usize, y: usize) -> Option<Tile> {
        self.tiles.get(y)?.get(x).cloned()
    }

    fn intersections(&self) -> usize {
        let mut alignment = 0;

        for (y, row) in self.tiles.iter().enumerate() {
            for (x, &tile) in row.iter().enumerate() {
                if x > 0
                    && y > 0
                    && tile == Tile::Scaffold
                    && self.get(x, y - 1) == Some(Tile::Scaffold)
                    && self.get(x, y + 1) == Some(Tile::Scaffold)
                    && self.get(x - 1, y) == Some(Tile::Scaffold)
                    && self.get(x + 1, y) == Some(Tile::Scaffold)
                {
                    alignment += x * y;
                }
            }
        }

        alignment
    }
}

pub fn calculate_alignment(program: &str) -> Result<usize, ProgramParseError> {
    let mut computer = Computer::new(program)?;
    let (_, receiver) = computer.get_io();

    let thread = thread::spawn(move || {
        computer.run();
    });

    let mut map = Map::new();

    for message in receiver {
        map.append(char::from(message as u8));
    }

    thread.join().unwrap();

    Ok(map.intersections())
}
