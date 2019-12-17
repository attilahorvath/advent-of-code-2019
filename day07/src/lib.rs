use std::sync::mpsc;
use std::thread;

use intcode::{Computer, Io, ValueType};

struct ChainedIo {
    input: mpsc::Receiver<ValueType>,
    outputs: Vec<mpsc::Sender<ValueType>>,
}

impl ChainedIo {
    fn new() -> (Self, mpsc::Sender<ValueType>) {
        let (sender, input) = mpsc::channel();

        (
            Self {
                input: input,
                outputs: vec![],
            },
            sender,
        )
    }

    fn attach_output(&mut self, output: mpsc::Sender<ValueType>) {
        self.outputs.push(output);
    }
}

impl Io for ChainedIo {
    fn send(&mut self, value: ValueType) {
        for sender in &self.outputs {
            sender.send(value).unwrap_or(());
        }
    }

    fn receive(&mut self) -> ValueType {
        self.input.recv().unwrap()
    }
}

fn permutations(array: &mut [ValueType]) -> Vec<Vec<ValueType>> {
    fn generate_permutation(
        array: &mut [ValueType],
        permutations: &mut Vec<Vec<ValueType>>,
        index: usize,
    ) {
        if index == array.len() - 1 {
            permutations.push(array.to_vec());
        }

        for i in index..array.len() {
            array.swap(i, index);
            generate_permutation(array, permutations, index + 1);
            array.swap(i, index);
        }
    }

    let mut permutations = vec![];

    generate_permutation(array, &mut permutations, 0);

    permutations
}

fn chain_computers(
    program: &str,
    count: usize,
    feedback: bool,
) -> (
    Vec<(Computer, mpsc::Sender<ValueType>)>,
    mpsc::Receiver<ValueType>,
) {
    let indices = (0..count).collect::<Vec<_>>();

    let mut chained_ios = indices.iter().map(|_| ChainedIo::new()).collect::<Vec<_>>();

    for window in indices.windows(2) {
        let sender = chained_ios[window[1]].1.clone();
        chained_ios[window[0]].0.attach_output(sender);
    }

    let (output, receiver) = if feedback {
        let (final_sender, final_receiver) = mpsc::channel();
        chained_ios[count - 1].0.attach_output(final_sender);

        (chained_ios[0].1.clone(), final_receiver)
    } else {
        mpsc::channel()
    };

    chained_ios[count - 1].0.attach_output(output);

    let computers = chained_ios
        .into_iter()
        .map(|(chained_io, sender)| {
            let mut computer = Computer::new(program).unwrap();
            computer.attach_io(Box::new(chained_io));

            (computer, sender)
        })
        .collect::<Vec<_>>();

    (computers, receiver)
}

pub fn highest_signal(program: &str, phases: &mut [ValueType], feedback: bool) -> ValueType {
    permutations(phases)
        .iter()
        .map(|permutation| {
            let (computers, receiver) = chain_computers(program, permutation.len(), feedback);

            for (c, &i) in computers.iter().zip(permutation.iter()) {
                c.1.send(i).unwrap();
            }

            computers[0].1.send(0).unwrap();

            let threads = computers
                .into_iter()
                .map(|(mut c, _)| {
                    thread::spawn(move || {
                        c.run();
                    })
                })
                .collect::<Vec<_>>();

            threads.into_iter().for_each(|t| t.join().unwrap());

            receiver.iter().last().unwrap()
        })
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highest_signal_1() {
        let program = "3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0";

        assert_eq!(
            43_210,
            highest_signal(&program, &mut [0, 1, 2, 3, 4], false)
        );
    }

    #[test]
    fn highest_signal_2() {
        let program = "3,23,3,24,1002,24,10,24,1002,23,-1,23,\
                       101,5,23,23,1,24,23,23,4,23,99,0,0";

        assert_eq!(
            54_321,
            highest_signal(&program, &mut [0, 1, 2, 3, 4], false)
        );
    }

    #[test]
    fn highest_signal_3() {
        let program = "3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,\
                       1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0";

        assert_eq!(
            65_210,
            highest_signal(&program, &mut [0, 1, 2, 3, 4], false)
        );
    }

    #[test]
    fn highest_signal_with_feedback_1() {
        let program = "3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,\
                       27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5";

        assert_eq!(
            139_629_729,
            highest_signal(&program, &mut [5, 6, 7, 8, 9], true)
        );
    }

    #[test]
    fn highest_signal_with_feedback_2() {
        let program = "3,52,1001,52,-5,52,3,53,1,52,56,54,1007,54,5,55,1005,55,26,1001,54,\
                       -5,54,1105,1,12,1,53,54,53,1008,54,0,55,1001,55,1,55,2,53,55,53,4,\
                       53,1001,56,-1,56,1005,56,6,99,0,0,0,0,10";

        assert_eq!(18_216, highest_signal(&program, &mut [5, 6, 7, 8, 9], true));
    }
}
