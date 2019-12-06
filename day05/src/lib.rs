use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy)]
enum ParameterMode {
    Position = 0,
    Immediate = 1,
}

#[derive(Debug, PartialEq)]
struct Memory {
    values: Vec<i32>,
    ip: usize,
}

impl Memory {
    fn new(values: &[i32]) -> Self {
        Memory {
            values: values.to_vec(),
            ip: 0,
        }
    }

    fn value_at_offset(&self, offset: usize, mode: ParameterMode) -> i32 {
        match mode {
            ParameterMode::Position => self.values[self.values[self.ip + offset] as usize],
            ParameterMode::Immediate => self.values[self.ip + offset],
        }
    }

    fn set_value_at_offset(&mut self, offset: usize, value: i32) {
        let index = self.values[self.ip + offset] as usize;

        self.values[index] = value;
    }

    fn current_opcode(&self) -> i32 {
        self.values[self.ip]
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

impl FromStr for Memory {
    type Err = ProgramParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values = s
            .split(",")
            .map(|value| value.parse::<i32>().map_err(|_| ProgramParseError))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Memory::new(&values))
    }
}

struct Io {
    input: i32,
    outputs: Vec<i32>,
}

impl Io {
    fn new() -> Self {
        Io {
            input: 0,
            outputs: Vec::new(),
        }
    }

    fn output(&mut self, value: i32) {
        println!("{}", value);

        self.outputs.push(value);
    }
}

#[derive(Clone, Copy)]
enum Opcode {
    AddCode = 1,
    MultiplyCode = 2,
    InputCode = 3,
    OutputCode = 4,
    JumpIfTrueCode = 5,
    JumpIfFalseCode = 6,
    LessThanCode = 7,
    EqualsCode = 8,
    HaltCode = 99,
}

enum OperationMode {
    None,
    Advance,
    Halt,
}

trait Operation {
    fn execute(&self, _memory: &mut Memory, _io: &mut Io) {}

    fn width(&self) -> usize {
        4
    }

    fn mode(&self) -> OperationMode {
        OperationMode::Advance
    }
}

struct Add(ParameterMode, ParameterMode);
struct Multiply(ParameterMode, ParameterMode);
struct Input;
struct Output(ParameterMode);
struct JumpIfTrue(ParameterMode, ParameterMode);
struct JumpIfFalse(ParameterMode, ParameterMode);
struct LessThan(ParameterMode, ParameterMode);
struct Equals(ParameterMode, ParameterMode);
struct Halt;
struct Nop;

impl Opcode {
    fn parse(opcode: i32) -> Box<dyn Operation> {
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
            x if x == Opcode::AddCode as i32 => Box::new(Add(a_mode, b_mode)),
            x if x == Opcode::MultiplyCode as i32 => Box::new(Multiply(a_mode, b_mode)),
            x if x == Opcode::InputCode as i32 => Box::new(Input),
            x if x == Opcode::OutputCode as i32 => Box::new(Output(a_mode)),
            x if x == Opcode::JumpIfTrueCode as i32 => Box::new(JumpIfTrue(a_mode, b_mode)),
            x if x == Opcode::JumpIfFalseCode as i32 => Box::new(JumpIfFalse(a_mode, b_mode)),
            x if x == Opcode::LessThanCode as i32 => Box::new(LessThan(a_mode, b_mode)),
            x if x == Opcode::EqualsCode as i32 => Box::new(Equals(a_mode, b_mode)),
            x if x == Opcode::HaltCode as i32 => Box::new(Halt),
            _ => Box::new(Nop),
        }
    }
}

impl Operation for Add {
    fn execute(&self, memory: &mut Memory, _io: &mut Io) {
        let a = memory.value_at_offset(1, self.0);
        let b = memory.value_at_offset(2, self.1);

        memory.set_value_at_offset(3, a + b);
    }
}

impl Operation for Multiply {
    fn execute(&self, memory: &mut Memory, _io: &mut Io) {
        let a = memory.value_at_offset(1, self.0);
        let b = memory.value_at_offset(2, self.1);

        memory.set_value_at_offset(3, a * b);
    }
}

impl Operation for Input {
    fn execute(&self, memory: &mut Memory, io: &mut Io) {
        memory.set_value_at_offset(1, io.input);
    }

    fn width(&self) -> usize {
        2
    }
}

impl Operation for Output {
    fn execute(&self, memory: &mut Memory, io: &mut Io) {
        io.output(memory.value_at_offset(1, self.0));
    }

    fn width(&self) -> usize {
        2
    }
}

impl Operation for JumpIfTrue {
    fn execute(&self, memory: &mut Memory, _io: &mut Io) {
        let a = memory.value_at_offset(1, self.0);
        let b = memory.value_at_offset(2, self.1);

        memory.ip = if a != 0 {
            b as usize
        } else {
            memory.ip + self.width()
        };
    }

    fn width(&self) -> usize {
        3
    }

    fn mode(&self) -> OperationMode {
        OperationMode::None
    }
}

impl Operation for JumpIfFalse {
    fn execute(&self, memory: &mut Memory, _io: &mut Io) {
        let a = memory.value_at_offset(1, self.0);
        let b = memory.value_at_offset(2, self.1);

        memory.ip = if a == 0 {
            b as usize
        } else {
            memory.ip + self.width()
        };
    }

    fn width(&self) -> usize {
        3
    }

    fn mode(&self) -> OperationMode {
        OperationMode::None
    }
}

impl Operation for LessThan {
    fn execute(&self, memory: &mut Memory, _io: &mut Io) {
        let a = memory.value_at_offset(1, self.0);
        let b = memory.value_at_offset(2, self.1);

        memory.set_value_at_offset(3, if a < b { 1 } else { 0 });
    }
}

impl Operation for Equals {
    fn execute(&self, memory: &mut Memory, _io: &mut Io) {
        let a = memory.value_at_offset(1, self.0);
        let b = memory.value_at_offset(2, self.1);

        memory.set_value_at_offset(3, if a == b { 1 } else { 0 });
    }
}

impl Operation for Halt {
    fn mode(&self) -> OperationMode {
        OperationMode::Halt
    }

    fn width(&self) -> usize {
        1
    }
}

impl Operation for Nop {
    fn width(&self) -> usize {
        1
    }
}

pub struct Computer {
    memory: Memory,
    io: Io,
}

impl Computer {
    pub fn new(program: &str) -> Result<Self, ProgramParseError> {
        Ok(Self {
            memory: program.parse()?,
            io: Io::new(),
        })
    }

    pub fn run(&mut self, input: i32) {
        self.io.input = input;
        self.memory.ip = 0;

        loop {
            let opcode = self.memory.current_opcode();
            let operation = Opcode::parse(opcode);

            operation.execute(&mut self.memory, &mut self.io);

            match operation.mode() {
                OperationMode::Advance => self.memory.ip += operation.width(),
                OperationMode::Halt => break,
                _ => (),
            }
        }
    }

    pub fn output(&self) -> i32 {
        self.memory.values[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_program() {
        let memory = "1,0,0,0,99".parse::<Memory>();

        assert_eq!(
            Ok(Memory {
                values: vec![1, 0, 0, 0, 99],
                ip: 0,
            }),
            memory
        );
    }

    #[test]
    fn addition() {
        let mut computer = Computer::new("1,0,0,0,99").unwrap();

        computer.run(1);

        assert_eq!(vec![2, 0, 0, 0, 99], computer.memory.values);
    }

    #[test]
    fn multiplication() {
        let mut computer = Computer::new("2,3,0,3,99").unwrap();

        computer.run(1);

        assert_eq!(vec![2, 3, 0, 6, 99], computer.memory.values);
    }

    #[test]
    fn large_multiplication() {
        let mut computer = Computer::new("2,4,4,5,99,0").unwrap();

        computer.run(1);

        assert_eq!(vec![2, 4, 4, 5, 99, 9801], computer.memory.values);
    }

    #[test]
    fn multiple_operations() {
        let mut computer = Computer::new("1,1,1,4,99,5,6,0,99").unwrap();

        computer.run(1);

        assert_eq!(vec![30, 1, 1, 4, 2, 5, 6, 0, 99], computer.memory.values);
    }

    #[test]
    fn io() {
        let mut computer = Computer::new("3,0,4,0,99").unwrap();

        computer.run(1);

        assert_eq!(vec![1], computer.io.outputs);
    }

    #[test]
    fn parameter_modes() {
        let mut computer = Computer::new("1002,4,3,4,33").unwrap();

        computer.run(1);

        assert_eq!(vec![1002, 4, 3, 4, 99], computer.memory.values);
    }

    #[test]
    fn negative_value() {
        let mut computer = Computer::new("1101,100,-1,4,0").unwrap();

        computer.run(1);

        assert_eq!(vec![1101, 100, -1, 4, 99], computer.memory.values);
    }

    #[test]
    fn equals_position_mode() {
        let program = "3,9,8,9,10,9,4,9,99,-1,8";

        let mut computer = Computer::new(&program).unwrap();

        computer.run(8);

        assert_eq!(vec![1], computer.io.outputs);

        computer = Computer::new(&program).unwrap();

        computer.run(10);

        assert_eq!(vec![0], computer.io.outputs);
    }

    #[test]
    fn less_than_position_mode() {
        let program = "3,9,7,9,10,9,4,9,99,-1,8";

        let mut computer = Computer::new(&program).unwrap();

        computer.run(6);

        assert_eq!(vec![1], computer.io.outputs);

        computer = Computer::new(&program).unwrap();

        computer.run(10);

        assert_eq!(vec![0], computer.io.outputs);
    }

    #[test]
    fn equals_immediate_mode() {
        let program = "3,3,1108,-1,8,3,4,3,99";

        let mut computer = Computer::new(&program).unwrap();

        computer.run(8);

        assert_eq!(vec![1], computer.io.outputs);

        computer = Computer::new(&program).unwrap();

        computer.run(10);

        assert_eq!(vec![0], computer.io.outputs);
    }

    #[test]
    fn less_than_immediate_mode() {
        let program = "3,3,1107,-1,8,3,4,3,99";

        let mut computer = Computer::new(&program).unwrap();

        computer.run(6);

        assert_eq!(vec![1], computer.io.outputs);

        computer = Computer::new(&program).unwrap();

        computer.run(10);

        assert_eq!(vec![0], computer.io.outputs);
    }

    #[test]
    fn jump_test_position_mode() {
        let program = "3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9";

        let mut computer = Computer::new(&program).unwrap();

        computer.run(0);

        assert_eq!(vec![0], computer.io.outputs);

        computer = Computer::new(&program).unwrap();

        computer.run(12);

        assert_eq!(vec![1], computer.io.outputs);
    }

    #[test]
    fn jump_test_immediate_mode() {
        let program = "3,3,1105,-1,9,1101,0,0,12,4,12,99,1";

        let mut computer = Computer::new(&program).unwrap();

        computer.run(0);

        assert_eq!(vec![0], computer.io.outputs);

        computer = Computer::new(&program).unwrap();

        computer.run(12);

        assert_eq!(vec![1], computer.io.outputs);
    }

    #[test]
    fn complex_program() {
        let program = "3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,\
                       1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,\
                       999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99";

        let mut computer = Computer::new(&program).unwrap();

        computer.run(6);

        assert_eq!(vec![999], computer.io.outputs);

        computer = Computer::new(&program).unwrap();

        computer.run(8);

        assert_eq!(vec![1000], computer.io.outputs);

        computer = Computer::new(&program).unwrap();

        computer.run(12);

        assert_eq!(vec![1001], computer.io.outputs);
    }
}
