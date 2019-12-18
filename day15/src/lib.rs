use std::collections::HashSet;
use std::sync::mpsc;
use std::thread;

use intcode::{Computer, ProgramParseError, ValueType};

#[derive(Clone, Copy, PartialEq)]
enum Command {
    North = 1,
    South = 2,
    West = 3,
    East = 4,
}

impl Command {
    fn apply(&self, position: (ValueType, ValueType)) -> (ValueType, ValueType) {
        match *self {
            Command::North => (position.0, position.1 - 1),
            Command::South => (position.0, position.1 + 1),
            Command::West => (position.0 - 1, position.1),
            Command::East => (position.0 + 1, position.1),
        }
    }

    fn reverse(&self) -> Command {
        match *self {
            Command::North => Command::South,
            Command::South => Command::North,
            Command::West => Command::East,
            Command::East => Command::West,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Status {
    WallHit = 0,
    Moved = 1,
    TargetFound = 2,
}

impl From<ValueType> for Status {
    fn from(value: ValueType) -> Status {
        match value {
            x if x == Status::WallHit as ValueType => Status::WallHit,
            x if x == Status::Moved as ValueType => Status::Moved,
            x if x == Status::TargetFound as ValueType => Status::TargetFound,
            _ => Status::WallHit,
        }
    }
}

pub struct RemoteControl {
    sender: mpsc::Sender<ValueType>,
    receiver: mpsc::Receiver<ValueType>,
    shutdown_button: mpsc::Sender<()>,
    explored: HashSet<(ValueType, ValueType)>,
}

impl RemoteControl {
    pub fn new(program: &str) -> Result<Self, ProgramParseError> {
        let mut computer = Computer::new(program)?;
        let (sender, receiver) = computer.get_io();
        let shutdown_button = computer.shutdown_button();

        thread::spawn(move || {
            computer.run();
        });

        Ok(Self {
            sender,
            receiver,
            shutdown_button,
            explored: HashSet::new(),
        })
    }

    fn explore_direction(
        &mut self,
        position: (ValueType, ValueType),
        command: Command,
        depth: usize,
    ) -> Option<usize> {
        let position = command.apply(position);

        if self.explored.contains(&position) {
            return None;
        }

        self.sender.send(command as ValueType).unwrap();

        match self.receiver.recv().unwrap().into() {
            Status::WallHit => None,
            Status::Moved => {
                if let Some(result) = self.explore(position, depth + 1) {
                    Some(result)
                } else {
                    self.sender.send(command.reverse() as ValueType).unwrap();
                    self.receiver.recv().unwrap();

                    None
                }
            }
            Status::TargetFound => Some(depth),
        }
    }

    fn explore(&mut self, position: (ValueType, ValueType), depth: usize) -> Option<usize> {
        self.explored.insert(position);

        for &command in &[Command::North, Command::South, Command::West, Command::East] {
            let result = self.explore_direction(position, command, depth);

            if result.is_some() {
                return result;
            }
        }

        None
    }

    fn max_depth_in_direction(
        &mut self,
        position: (ValueType, ValueType),
        command: Command,
        depth: usize,
    ) -> usize {
        let position = command.apply(position);

        if self.explored.contains(&position) {
            return depth;
        }

        self.sender.send(command as ValueType).unwrap();

        match self.receiver.recv().unwrap().into() {
            Status::WallHit => depth,
            _ => {
                let result = self.max_depth(position, depth + 1);

                self.sender.send(command.reverse() as ValueType).unwrap();
                self.receiver.recv().unwrap();

                result
            }
        }
    }

    fn max_depth(&mut self, position: (ValueType, ValueType), depth: usize) -> usize {
        self.explored.insert(position);

        [Command::North, Command::South, Command::West, Command::East]
            .iter()
            .map(|&command| self.max_depth_in_direction(position, command, depth))
            .max()
            .unwrap_or(depth)
    }

    pub fn find_target(&mut self) -> usize {
        self.explore((0, 0), 1).unwrap_or(0)
    }

    pub fn find_max_depth(&mut self) -> usize {
        self.explored.clear();

        self.max_depth((0, 0), 0)
    }
}

impl Drop for RemoteControl {
    fn drop(&mut self) {
        self.shutdown_button.send(()).unwrap();
        self.sender.send(0).unwrap();
    }
}
