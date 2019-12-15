use std::collections::HashMap;
use std::fmt;
use std::thread;

use intcode::{Computer, ProgramParseError, ValueType};

#[derive(Clone, Copy, PartialEq)]
pub enum Color {
    Black = 0,
    White = 1,
}

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn turn_left(&mut self) {
        *self = match self {
            Direction::Up => Direction::Left,
            Direction::Down => Direction::Right,
            Direction::Left => Direction::Down,
            Direction::Right => Direction::Up,
        };
    }

    fn turn_right(&mut self) {
        *self = match self {
            Direction::Up => Direction::Right,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Right => Direction::Down,
        };
    }
}

struct Panel {
    color: Color,
}

impl Panel {
    fn new() -> Self {
        Self {
            color: Color::Black,
        }
    }

    fn with_color(color: Color) -> Self {
        Self { color }
    }
}

pub struct Hull {
    panels: HashMap<(i32, i32), Panel>,
}

impl Hull {
    pub fn new() -> Self {
        Self {
            panels: HashMap::new(),
        }
    }
}

impl fmt::Display for Hull {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let min_x = self.panels.keys().map(|k| k.0).min().unwrap_or(0);
        let max_x = self.panels.keys().map(|k| k.0).max().unwrap_or(0);
        let min_y = self.panels.keys().map(|k| k.1).min().unwrap_or(0);
        let max_y = self.panels.keys().map(|k| k.1).max().unwrap_or(0);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let color = self.panels.get(&(x, y)).unwrap_or(&Panel::new()).color;
                write!(f, "{}", if color == Color::White { '#' } else { ' ' })?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

pub struct Robot {
    position: (i32, i32),
    direction: Direction,
}

impl Robot {
    pub fn new() -> Self {
        Self {
            position: (0, 0),
            direction: Direction::Up,
        }
    }

    pub fn run(
        &mut self,
        program: &str,
        hull: &mut Hull,
        starting_color: Color,
    ) -> Result<usize, ProgramParseError> {
        let mut computer = Computer::new(program)?;
        let (sender, receiver) = computer.get_io();

        let thread = thread::spawn(move || {
            computer.run();
        });

        hull.panels
            .insert(self.position, Panel::with_color(starting_color));

        loop {
            let panel = hull.panels.entry(self.position).or_insert(Panel::new());

            sender.send(panel.color as ValueType).unwrap();

            if let Ok(color) = receiver.recv() {
                match color {
                    x if x == Color::Black as ValueType => panel.color = Color::Black,
                    x if x == Color::White as ValueType => panel.color = Color::White,
                    _ => (),
                }
            } else {
                break;
            }

            if receiver.recv().unwrap() == 0 {
                self.direction.turn_left();
            } else {
                self.direction.turn_right();
            }

            self.step();
        }

        thread.join().unwrap();

        Ok(hull.panels.len())
    }

    fn step(&mut self) {
        self.position = match self.direction {
            Direction::Up => (self.position.0, self.position.1 - 1),
            Direction::Down => (self.position.0, self.position.1 + 1),
            Direction::Left => (self.position.0 - 1, self.position.1),
            Direction::Right => (self.position.0 + 1, self.position.1),
        };
    }
}
