use std::error::Error;
use std::fmt;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;

#[derive(Clone, Copy)]
enum ParameterMode {
    Position = 0,
    Immediate = 1,
}

#[derive(Debug, PartialEq)]
struct Program {
    values: Vec<i32>,
}

impl Program {
    fn new(values: &[i32]) -> Self {
        Self {
            values: values.to_vec(),
        }
    }
}

struct Memory {
    values: Vec<i32>,
    ip: usize,
}

impl Memory {
    fn new() -> Self {
        Memory {
            values: Vec::new(),
            ip: 0,
        }
    }

    fn load(&mut self, program: &Program) {
        self.values = program.values.clone();
        self.ip = 0;
    }

    fn advance(&mut self, length: usize) -> &[i32] {
        let values = &self.values[self.ip..self.ip + length];

        self.ip += length;

        values
    }

    fn get(&self, address: i32) -> i32 {
        self.values[address as usize]
    }

    fn set(&mut self, address: i32, value: i32) {
        self.values[address as usize] = value;
    }

    fn jump(&mut self, address: i32) {
        self.ip = address as usize;
    }
}

#[derive(Debug, PartialEq)]
pub struct ProgramParseError;

impl fmt::Display for ProgramParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unable to parse program")
    }
}

impl Error for ProgramParseError {}

impl FromStr for Program {
    type Err = ProgramParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values = s
            .split(",")
            .map(|value| value.parse::<i32>().map_err(|_| ProgramParseError))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Program::new(&values))
    }
}

struct Io {
    input: Option<mpsc::Receiver<i32>>,
    output: Option<mpsc::Sender<i32>>,
    final_output: Option<mpsc::Sender<i32>>,
}

impl Io {
    fn new() -> Self {
        Io {
            input: None,
            output: None,
            final_output: None,
        }
    }

    fn send(&self, value: i32) {
        if let Some(sender) = &self.output {
            sender.send(value).unwrap_or(());
        }

        if let Some(sender) = &self.final_output {
            sender.send(value).unwrap();
        }
    }

    fn receive(&self) -> i32 {
        if let Some(receiver) = &self.input {
            receiver.recv().unwrap()
        } else {
            0
        }
    }
}

#[derive(Clone, Copy)]
enum Opcode {
    Add = 1,
    Multiply = 2,
    Input = 3,
    Output = 4,
    JumpIfTrue = 5,
    JumpIfFalse = 6,
    LessThan = 7,
    Equals = 8,
    Halt = 99,
}

struct Operation {
    parameter_modes: Vec<ParameterMode>,
    operation: fn(&[i32], &mut Memory, &mut Io),
    halt: bool,
}

impl Operation {
    fn new(operation: fn(&[i32], &mut Memory, &mut Io), parameter_modes: &[ParameterMode]) -> Self {
        Self {
            parameter_modes: parameter_modes.to_vec(),
            operation,
            halt: false,
        }
    }

    fn halt(mut self) -> Self {
        self.halt = true;

        self
    }

    fn execute(&self, memory: &mut Memory, io: &mut Io) -> bool {
        let parameters = memory.advance(self.parameter_modes.len()).to_vec();

        let values = parameters
            .iter()
            .zip(self.parameter_modes.iter())
            .map(|(&param, mode)| match mode {
                ParameterMode::Position => memory.get(param),
                ParameterMode::Immediate => param,
            })
            .collect::<Vec<_>>();

        (self.operation)(&values, memory, io);

        !self.halt
    }
}

fn add(values: &[i32], memory: &mut Memory, _io: &mut Io) {
    memory.set(values[2], values[0] + values[1]);
}

fn multiply(values: &[i32], memory: &mut Memory, _io: &mut Io) {
    memory.set(values[2], values[0] * values[1]);
}

fn input(values: &[i32], memory: &mut Memory, io: &mut Io) {
    memory.set(values[0], io.receive());
}

fn output(values: &[i32], _memory: &mut Memory, io: &mut Io) {
    io.send(values[0]);
}

fn jump_if_true(values: &[i32], memory: &mut Memory, _io: &mut Io) {
    if values[0] != 0 {
        memory.jump(values[1]);
    }
}

fn jump_if_false(values: &[i32], memory: &mut Memory, _io: &mut Io) {
    if values[0] == 0 {
        memory.jump(values[1]);
    }
}

fn less_than(values: &[i32], memory: &mut Memory, _io: &mut Io) {
    memory.set(values[2], if values[0] < values[1] { 1 } else { 0 });
}

fn equals(values: &[i32], memory: &mut Memory, _io: &mut Io) {
    memory.set(values[2], if values[0] == values[1] { 1 } else { 0 });
}

fn nop(_values: &[i32], _memory: &mut Memory, _io: &mut Io) {}

impl Opcode {
    fn parse(opcode: i32) -> Operation {
        let a_mode = if opcode % 1000 >= 100 {
            ParameterMode::Immediate
        } else {
            ParameterMode::Position
        };

        let b_mode = if opcode % 10_000 >= 1000 {
            ParameterMode::Immediate
        } else {
            ParameterMode::Position
        };

        match opcode % 100 {
            x if x == Opcode::Add as i32 => {
                Operation::new(add, &[a_mode, b_mode, ParameterMode::Immediate])
            }
            x if x == Opcode::Multiply as i32 => {
                Operation::new(multiply, &[a_mode, b_mode, ParameterMode::Immediate])
            }
            x if x == Opcode::Input as i32 => Operation::new(input, &[ParameterMode::Immediate]),
            x if x == Opcode::Output as i32 => Operation::new(output, &[a_mode]),
            x if x == Opcode::JumpIfTrue as i32 => Operation::new(jump_if_true, &[a_mode, b_mode]),
            x if x == Opcode::JumpIfFalse as i32 => {
                Operation::new(jump_if_false, &[a_mode, b_mode])
            }
            x if x == Opcode::LessThan as i32 => {
                Operation::new(less_than, &[a_mode, b_mode, ParameterMode::Immediate])
            }
            x if x == Opcode::Equals as i32 => {
                Operation::new(equals, &[a_mode, b_mode, ParameterMode::Immediate])
            }
            x if x == Opcode::Halt as i32 => Operation::new(nop, &[]).halt(),
            _ => Operation::new(nop, &[]),
        }
    }
}

pub struct Computer {
    program: Program,
    memory: Memory,
    io: Io,
}

impl Computer {
    pub fn new(program: &str) -> Result<Self, ProgramParseError> {
        Ok(Self {
            program: program.parse()?,
            memory: Memory::new(),
            io: Io::new(),
        })
    }

    pub fn attach_input(&mut self, input: mpsc::Receiver<i32>) {
        self.io.input = Some(input);
    }

    pub fn attach_output(&mut self, output: mpsc::Sender<i32>) {
        self.io.output = Some(output);
    }

    pub fn attach_final_output(&mut self, output: mpsc::Sender<i32>) {
        self.io.final_output = Some(output);
    }

    pub fn get_io(&mut self) -> (mpsc::Sender<i32>, mpsc::Receiver<i32>) {
        let (sender, input) = mpsc::channel();
        let (output, receiver) = mpsc::channel();

        self.io.input = Some(input);
        self.io.output = Some(output);

        (sender, receiver)
    }

    pub fn run(&mut self) {
        self.memory.load(&self.program);

        loop {
            let opcode = self.memory.advance(1)[0];
            let operation = Opcode::parse(opcode);

            if !operation.execute(&mut self.memory, &mut self.io) {
                break;
            }
        }
    }
}

fn all_permutations(array: &mut [i32]) -> Vec<Vec<i32>> {
    let mut permutations = vec![];

    generate_permutation(array, &mut permutations, 0);

    permutations
}

fn generate_permutation(array: &mut [i32], permutations: &mut Vec<Vec<i32>>, index: usize) {
    if index == array.len() - 1 {
        permutations.push(array.to_vec());
    }

    for i in index..array.len() {
        array.swap(i, index);
        generate_permutation(array, permutations, index + 1);
        array.swap(i, index);
    }
}

fn chain_computers(
    program: &str,
    count: usize,
    feedback: bool,
) -> (Vec<(Computer, mpsc::Sender<i32>)>, mpsc::Receiver<i32>) {
    let indices = (0..count).collect::<Vec<_>>();

    let mut computers = indices
        .iter()
        .map(|_| {
            let mut computer = Computer::new(program).unwrap();
            let (sender, input) = mpsc::channel();
            computer.attach_input(input);
            (computer, sender)
        })
        .collect::<Vec<_>>();

    for window in indices.windows(2) {
        let sender = computers[window[1]].1.clone();
        computers[window[0]].0.attach_output(sender);
    }

    let (output, receiver) = if feedback {
        let (final_sender, final_receiver) = mpsc::channel();
        computers[count - 1].0.attach_final_output(final_sender);

        (computers[0].1.clone(), final_receiver)
    } else {
        mpsc::channel()
    };

    computers[count - 1].0.attach_output(output);

    (computers, receiver)
}

pub fn highest_signal(program: &str, phases: &mut [i32], feedback: bool) -> i32 {
    all_permutations(phases)
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
    fn parse_program() {
        let program = "1,0,0,0,99".parse::<Program>();

        assert_eq!(
            Ok(Program {
                values: vec![1, 0, 0, 0, 99],
            }),
            program
        );
    }

    #[test]
    fn addition() {
        let mut computer = Computer::new("1,0,0,0,99").unwrap();

        computer.run();

        assert_eq!(vec![2, 0, 0, 0, 99], computer.memory.values);
    }

    #[test]
    fn multiplication() {
        let mut computer = Computer::new("2,3,0,3,99").unwrap();

        computer.run();

        assert_eq!(vec![2, 3, 0, 6, 99], computer.memory.values);
    }

    #[test]
    fn large_multiplication() {
        let mut computer = Computer::new("2,4,4,5,99,0").unwrap();

        computer.run();

        assert_eq!(vec![2, 4, 4, 5, 99, 9801], computer.memory.values);
    }

    #[test]
    fn multiple_operations() {
        let mut computer = Computer::new("1,1,1,4,99,5,6,0,99").unwrap();

        computer.run();

        assert_eq!(vec![30, 1, 1, 4, 2, 5, 6, 0, 99], computer.memory.values);
    }

    #[test]
    fn io() {
        let mut computer = Computer::new("3,0,4,0,99").unwrap();

        let (sender, receiver) = computer.get_io();

        sender.send(1).unwrap();

        computer.run();

        assert_eq!(1, receiver.recv().unwrap());
    }

    #[test]
    fn parameter_modes() {
        let mut computer = Computer::new("1002,4,3,4,33").unwrap();

        computer.run();

        assert_eq!(vec![1002, 4, 3, 4, 99], computer.memory.values);
    }

    #[test]
    fn negative_value() {
        let mut computer = Computer::new("1101,100,-1,4,0").unwrap();

        computer.run();

        assert_eq!(vec![1101, 100, -1, 4, 99], computer.memory.values);
    }

    #[test]
    fn equals_position_mode() {
        let mut computer = Computer::new("3,9,8,9,10,9,4,9,99,-1,8").unwrap();

        let (sender, receiver) = computer.get_io();

        sender.send(8).unwrap();

        computer.run();

        assert_eq!(1, receiver.recv().unwrap());

        sender.send(10).unwrap();

        computer.run();

        assert_eq!(0, receiver.recv().unwrap());
    }

    #[test]
    fn less_than_position_mode() {
        let mut computer = Computer::new("3,9,7,9,10,9,4,9,99,-1,8").unwrap();

        let (sender, receiver) = computer.get_io();

        sender.send(6).unwrap();

        computer.run();

        assert_eq!(1, receiver.recv().unwrap());

        sender.send(10).unwrap();

        computer.run();

        assert_eq!(0, receiver.recv().unwrap());
    }

    #[test]
    fn equals_immediate_mode() {
        let mut computer = Computer::new("3,3,1108,-1,8,3,4,3,99").unwrap();

        let (sender, receiver) = computer.get_io();

        sender.send(8).unwrap();

        computer.run();

        assert_eq!(1, receiver.recv().unwrap());

        sender.send(10).unwrap();

        computer.run();

        assert_eq!(0, receiver.recv().unwrap());
    }

    #[test]
    fn less_than_immediate_mode() {
        let mut computer = Computer::new("3,3,1107,-1,8,3,4,3,99").unwrap();

        let (sender, receiver) = computer.get_io();

        sender.send(6).unwrap();

        computer.run();

        assert_eq!(1, receiver.recv().unwrap());

        sender.send(10).unwrap();

        computer.run();

        assert_eq!(0, receiver.recv().unwrap());
    }

    #[test]
    fn jump_test_position_mode() {
        let mut computer = Computer::new("3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9").unwrap();

        let (sender, receiver) = computer.get_io();

        sender.send(0).unwrap();

        computer.run();

        assert_eq!(0, receiver.recv().unwrap());

        sender.send(12).unwrap();

        computer.run();

        assert_eq!(1, receiver.recv().unwrap());
    }

    #[test]
    fn jump_test_immediate_mode() {
        let mut computer = Computer::new("3,3,1105,-1,9,1101,0,0,12,4,12,99,1").unwrap();

        let (sender, receiver) = computer.get_io();

        sender.send(0).unwrap();

        computer.run();

        assert_eq!(0, receiver.recv().unwrap());

        sender.send(12).unwrap();

        computer.run();

        assert_eq!(1, receiver.recv().unwrap());
    }

    #[test]
    fn complex_program() {
        let program = "3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,\
                       1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,\
                       999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99";

        let mut computer = Computer::new(&program).unwrap();

        let (sender, receiver) = computer.get_io();

        sender.send(6).unwrap();

        computer.run();

        assert_eq!(999, receiver.recv().unwrap());

        sender.send(8).unwrap();

        computer.run();

        assert_eq!(1000, receiver.recv().unwrap());

        sender.send(12).unwrap();

        computer.run();

        assert_eq!(1001, receiver.recv().unwrap());
    }

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
