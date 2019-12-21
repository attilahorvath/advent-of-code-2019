use std::sync::mpsc;

use intcode::{Computer, ProgramParseError, ValueType};

pub struct Beam {
    computer: Computer,
    sender: mpsc::Sender<ValueType>,
    receiver: mpsc::Receiver<ValueType>,
}

impl Beam {
    pub fn new(program: &str) -> Result<Self, ProgramParseError> {
        let mut computer = Computer::new(program)?;
        let (sender, receiver) = computer.get_io();

        Ok(Self {
            computer,
            sender,
            receiver,
        })
    }

    fn position_affected(&mut self, x: ValueType, y: ValueType) -> bool {
        self.sender.send(x).unwrap();
        self.sender.send(y).unwrap();

        self.computer.run();

        self.receiver.recv().unwrap() == 1
    }

    pub fn area_affected(&mut self, size: ValueType) -> ValueType {
        (0..size)
            .map(|y| (0..size).filter(|&x| self.position_affected(x, y)).count() as ValueType)
            .sum()
    }

    pub fn find_square(&mut self, size: ValueType) -> (ValueType, ValueType) {
        let mut beam_slice = (0, 0);

        for x in 0.. {
            let affected = self.position_affected(x, size);

            if affected && beam_slice.0 == 0 {
                beam_slice.0 = x;
            } else if !affected && beam_slice.0 > 0 && beam_slice.1 == 0 {
                beam_slice.1 = x;
                break;
            }
        }

        for y in size + 1.. {
            beam_slice = (
                (beam_slice.0..)
                    .find(|&x| self.position_affected(x, y))
                    .unwrap(),
                (beam_slice.1..)
                    .find(|&x| !self.position_affected(x, y))
                    .unwrap()
                    - 1,
            );

            if beam_slice.1 - beam_slice.0 >= size {
                let start = beam_slice.1 - (size - 1);

                if self.position_affected(start, y + (size - 1)) {
                    return (start, y);
                }
            }
        }

        (0, 0)
    }
}
