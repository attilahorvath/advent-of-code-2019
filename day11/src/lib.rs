use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;

const MEMORY_SIZE: usize = 4096;
type ValueType = i64;

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

fn all_permutations(array: &mut [ValueType]) -> Vec<Vec<ValueType>> {
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

pub fn highest_signal(program: &str, phases: &mut [ValueType], feedback: bool) -> ValueType {
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
