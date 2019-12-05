enum Opcode {
    AddCode = 1,
    MultiplyCode = 2,
    HaltCode = 99,
}

trait Operation {
    fn execute(&self, _memory: &mut [i32], _ip: usize) -> bool {
        true
    }

    fn width(&self) -> usize {
        4
    }
}

struct Add;
struct Multiply;
struct Halt;
struct Nop;

impl Opcode {
    fn parse(opcode: i32) -> Box<dyn Operation> {
        match opcode {
            x if x == Opcode::AddCode as i32 => Box::new(Add),
            x if x == Opcode::MultiplyCode as i32 => Box::new(Multiply),
            x if x == Opcode::HaltCode as i32 => Box::new(Halt),
            _ => Box::new(Nop),
        }
    }
}

impl Operation for Add {
    fn execute(&self, memory: &mut [i32], ip: usize) -> bool {
        let a = memory[memory[ip + 1] as usize];
        let b = memory[memory[ip + 2] as usize];
        let t = memory[ip + 3] as usize;

        memory[t] = a + b;

        true
    }
}

impl Operation for Multiply {
    fn execute(&self, memory: &mut [i32], ip: usize) -> bool {
        let a = memory[memory[ip + 1] as usize];
        let b = memory[memory[ip + 2] as usize];
        let t = memory[ip + 3] as usize;

        memory[t] = a * b;

        true
    }
}

impl Operation for Halt {
    fn execute(&self, _memory: &mut [i32], _ip: usize) -> bool {
        false
    }
}

impl Operation for Nop {
    fn width(&self) -> usize {
        1
    }
}

pub struct Computer {
    memory: Vec<i32>,
    ip: usize,
}

impl Computer {
    pub fn new(memory: &[i32]) -> Self {
        Self {
            memory: memory.to_vec(),
            ip: 0,
        }
    }

    pub fn with_inputs(mut self, noun: i32, verb: i32) -> Self {
        self.memory[1] = noun;
        self.memory[2] = verb;

        self
    }

    pub fn run(&mut self) {
        self.ip = 0;

        loop {
            let opcode = self.memory[self.ip];
            let operation = Opcode::parse(opcode);

            if !operation.execute(&mut self.memory, self.ip) {
                break;
            }

            self.ip += operation.width();
        }
    }

    pub fn output(&self) -> i32 {
        self.memory[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addition() {
        let mut computer = Computer::new(&[1, 0, 0, 0, 99]);

        computer.run();

        assert_eq!(vec![2, 0, 0, 0, 99], computer.memory);
    }

    #[test]
    fn multiplication() {
        let mut computer = Computer::new(&vec![2, 3, 0, 3, 99]);

        computer.run();

        assert_eq!(vec![2, 3, 0, 6, 99], computer.memory);
    }

    #[test]
    fn large_multiplication() {
        let mut computer = Computer::new(&vec![2, 4, 4, 5, 99, 0]);

        computer.run();

        assert_eq!(vec![2, 4, 4, 5, 99, 9801], computer.memory);
    }

    #[test]
    fn multiple_operations() {
        let mut computer = Computer::new(&vec![1, 1, 1, 4, 99, 5, 6, 0, 99]);

        computer.run();

        assert_eq!(vec![30, 1, 1, 4, 2, 5, 6, 0, 99], computer.memory);
    }
}
