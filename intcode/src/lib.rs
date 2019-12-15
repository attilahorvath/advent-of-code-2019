use std::error::Error;
use std::fmt;
use std::str::FromStr;
use std::sync::mpsc;

const MEMORY_SIZE: usize = 4096;
pub type ValueType = i64;

#[derive(Clone, Copy)]
enum ParameterMode {
    Position = 0,
    Immediate = 1,
    Relative = 2,
}

impl ParameterMode {
    fn parse_nth_digit(value: ValueType, n: u32) -> ParameterMode {
        let digit = value / (10 as ValueType).pow(n) % 10;

        match digit {
            x if x == ParameterMode::Position as ValueType => ParameterMode::Position,
            x if x == ParameterMode::Immediate as ValueType => ParameterMode::Immediate,
            x if x == ParameterMode::Relative as ValueType => ParameterMode::Relative,
            _ => ParameterMode::Position,
        }
    }
}

#[derive(Clone, Copy)]
struct Parameter {
    value: ValueType,
    mode: ParameterMode,
}

impl Parameter {
    fn new(value: ValueType, mode: ParameterMode) -> Self {
        Self { value, mode }
    }
}

#[derive(Debug, PartialEq)]
struct Program {
    values: Vec<ValueType>,
}

impl Program {
    fn new(values: &[ValueType]) -> Self {
        Self {
            values: values.to_vec(),
        }
    }
}

struct Memory {
    values: Vec<ValueType>,
    ip: usize,
    relative_base: ValueType,
}

impl Memory {
    fn new(size: usize) -> Self {
        Memory {
            values: vec![0; size],
            ip: 0,
            relative_base: 0,
        }
    }

    fn load(&mut self, program: &Program) {
        self.values.clear();
        self.values.resize(self.values.capacity(), 0);

        self.values
            .splice(..program.values.len(), program.values.clone());

        self.ip = 0;
        self.relative_base = 0;
    }

    fn load_values(&mut self, index: usize, values: &[ValueType]) {
        self.values
            .splice(index..index + values.len(), values.to_vec());
    }

    fn advance(&mut self, length: usize) -> &[ValueType] {
        let values = &self.values[self.ip..self.ip + length];

        self.ip += length;

        values
    }

    fn advance_relative_base(&mut self, amount: ValueType) {
        self.relative_base += amount;
    }

    fn get(&self, parameter: Parameter) -> ValueType {
        match parameter.mode {
            ParameterMode::Position => self.values[parameter.value as usize],
            ParameterMode::Immediate => parameter.value,
            ParameterMode::Relative => self.values[(self.relative_base + parameter.value) as usize],
        }
    }

    fn set(&mut self, parameter: Parameter, value: ValueType) {
        let position = match parameter.mode {
            ParameterMode::Position => &mut self.values[parameter.value as usize],
            ParameterMode::Immediate => &mut self.values[parameter.value as usize],
            ParameterMode::Relative => {
                &mut self.values[(self.relative_base + parameter.value) as usize]
            }
        };

        *position = value;
    }

    fn jump(&mut self, address: ValueType) {
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
            .map(|value| value.parse::<ValueType>().map_err(|_| ProgramParseError))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Program::new(&values))
    }
}

struct Io {
    input: Option<mpsc::Receiver<ValueType>>,
    output: Option<mpsc::Sender<ValueType>>,
    final_output: Option<mpsc::Sender<ValueType>>,
}

impl Io {
    fn new() -> Self {
        Io {
            input: None,
            output: None,
            final_output: None,
        }
    }

    fn send(&self, value: ValueType) {
        if let Some(sender) = &self.output {
            sender.send(value).unwrap_or(());
        }

        if let Some(sender) = &self.final_output {
            sender.send(value).unwrap();
        }
    }

    fn receive(&self) -> ValueType {
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
    AdjustRelativeBase = 9,
    Halt = 99,
}

struct Operation {
    parameter_modes: Vec<ParameterMode>,
    operation: fn(&[Parameter], &mut Memory, &mut Io),
    halt: bool,
}

impl Operation {
    fn new(
        operation: fn(&[Parameter], &mut Memory, &mut Io),
        parameter_modes: &[ParameterMode],
    ) -> Self {
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
        let parameters = memory
            .advance(self.parameter_modes.len())
            .iter()
            .zip(self.parameter_modes.iter())
            .map(|(&value, &mode)| Parameter::new(value, mode))
            .collect::<Vec<_>>();

        (self.operation)(&parameters, memory, io);

        !self.halt
    }
}

fn add(parameters: &[Parameter], memory: &mut Memory, _io: &mut Io) {
    memory.set(
        parameters[2],
        memory.get(parameters[0]) + memory.get(parameters[1]),
    );
}

fn multiply(parameters: &[Parameter], memory: &mut Memory, _io: &mut Io) {
    memory.set(
        parameters[2],
        memory.get(parameters[0]) * memory.get(parameters[1]),
    );
}

fn input(parameters: &[Parameter], memory: &mut Memory, io: &mut Io) {
    memory.set(parameters[0], io.receive());
}

fn output(parameters: &[Parameter], memory: &mut Memory, io: &mut Io) {
    io.send(memory.get(parameters[0]));
}

fn jump_if_true(parameters: &[Parameter], memory: &mut Memory, _io: &mut Io) {
    if memory.get(parameters[0]) != 0 {
        memory.jump(memory.get(parameters[1]));
    }
}

fn jump_if_false(parameters: &[Parameter], memory: &mut Memory, _io: &mut Io) {
    if memory.get(parameters[0]) == 0 {
        memory.jump(memory.get(parameters[1]));
    }
}

fn less_than(parameters: &[Parameter], memory: &mut Memory, _io: &mut Io) {
    memory.set(
        parameters[2],
        if memory.get(parameters[0]) < memory.get(parameters[1]) {
            1
        } else {
            0
        },
    );
}

fn equals(parameters: &[Parameter], memory: &mut Memory, _io: &mut Io) {
    memory.set(
        parameters[2],
        if memory.get(parameters[0]) == memory.get(parameters[1]) {
            1
        } else {
            0
        },
    );
}

fn adjust_relative_base(parameters: &[Parameter], memory: &mut Memory, _io: &mut Io) {
    memory.advance_relative_base(memory.get(parameters[0]));
}

fn nop(_parameters: &[Parameter], _memory: &mut Memory, _io: &mut Io) {}

impl Opcode {
    fn parse(opcode: ValueType) -> Operation {
        let modes = (2..=4)
            .map(|n| ParameterMode::parse_nth_digit(opcode, n))
            .collect::<Vec<_>>();

        match opcode % 100 {
            x if x == Opcode::Add as ValueType => Operation::new(add, &modes),
            x if x == Opcode::Multiply as ValueType => Operation::new(multiply, &modes),
            x if x == Opcode::Input as ValueType => Operation::new(input, &modes[0..1]),
            x if x == Opcode::Output as ValueType => Operation::new(output, &modes[0..1]),
            x if x == Opcode::JumpIfTrue as ValueType => Operation::new(jump_if_true, &modes[0..2]),
            x if x == Opcode::JumpIfFalse as ValueType => {
                Operation::new(jump_if_false, &modes[0..2])
            }
            x if x == Opcode::LessThan as ValueType => Operation::new(less_than, &modes[0..3]),
            x if x == Opcode::Equals as ValueType => Operation::new(equals, &modes[0..3]),
            x if x == Opcode::AdjustRelativeBase as ValueType => {
                Operation::new(adjust_relative_base, &modes[0..1])
            }
            x if x == Opcode::Halt as ValueType => Operation::new(nop, &[]).halt(),
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
            memory: Memory::new(MEMORY_SIZE),
            io: Io::new(),
        })
    }

    pub fn attach_input(&mut self, input: mpsc::Receiver<ValueType>) {
        self.io.input = Some(input);
    }

    pub fn attach_output(&mut self, output: mpsc::Sender<ValueType>) {
        self.io.output = Some(output);
    }

    pub fn attach_final_output(&mut self, output: mpsc::Sender<ValueType>) {
        self.io.final_output = Some(output);
    }

    pub fn get_io(&mut self) -> (mpsc::Sender<ValueType>, mpsc::Receiver<ValueType>) {
        let (sender, input) = mpsc::channel();
        let (output, receiver) = mpsc::channel();

        self.io.input = Some(input);
        self.io.output = Some(output);

        (sender, receiver)
    }

    pub fn dma(&mut self, position: usize) -> &mut ValueType {
        &mut self.memory.values[position]
    }

    pub fn run(&mut self) {
        self.memory.load(&self.program);

        self.execute();
    }

    pub fn run_with_values(&mut self, index: usize, values: &[ValueType]) {
        self.memory.load(&self.program);
        self.memory.load_values(index, values);

        self.execute();
    }

    fn execute(&mut self) {
        loop {
            let opcode = self.memory.advance(1)[0];
            let operation = Opcode::parse(opcode);

            if !operation.execute(&mut self.memory, &mut self.io) {
                break;
            }
        }
    }
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

        assert_eq!(vec![2, 0, 0, 0, 99], &computer.memory.values[..5]);
    }

    #[test]
    fn multiplication() {
        let mut computer = Computer::new("2,3,0,3,99").unwrap();

        computer.run();

        assert_eq!(vec![2, 3, 0, 6, 99], &computer.memory.values[..5]);
    }

    #[test]
    fn large_multiplication() {
        let mut computer = Computer::new("2,4,4,5,99,0").unwrap();

        computer.run();

        assert_eq!(vec![2, 4, 4, 5, 99, 9801], &computer.memory.values[..6]);
    }

    #[test]
    fn multiple_operations() {
        let mut computer = Computer::new("1,1,1,4,99,5,6,0,99").unwrap();

        computer.run();

        assert_eq!(
            vec![30, 1, 1, 4, 2, 5, 6, 0, 99],
            &computer.memory.values[..9]
        );
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

        assert_eq!(vec![1002, 4, 3, 4, 99], &computer.memory.values[..5]);
    }

    #[test]
    fn negative_value() {
        let mut computer = Computer::new("1101,100,-1,4,0").unwrap();

        computer.run();

        assert_eq!(vec![1101, 100, -1, 4, 99], &computer.memory.values[..5]);
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
    fn quine() {
        let program = "109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99";
        let values = program.parse::<Program>().unwrap().values;

        let mut computer = Computer::new(&program).unwrap();

        let (_, receiver) = computer.get_io();

        computer.run();

        assert_eq!(
            values,
            receiver.iter().take(values.len()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn multiply_big_integers() {
        let mut computer = Computer::new("1102,34915192,34915192,7,4,7,99,0").unwrap();

        let (_, receiver) = computer.get_io();

        computer.run();

        assert_eq!(1_219_070_632_396_864, receiver.recv().unwrap());
    }

    #[test]
    fn output_big_integer() {
        let mut computer = Computer::new("104,1125899906842624,99").unwrap();

        let (_, receiver) = computer.get_io();

        computer.run();

        assert_eq!(1_125_899_906_842_624, receiver.recv().unwrap());
    }
}
