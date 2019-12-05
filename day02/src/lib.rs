const ADD: i32 = 1;
const MULTIPLY: i32 = 2;
const HALT: i32 = 99;

pub fn run(memory: &mut [i32]) {
    let mut ip = 0;

    loop {
        let opcode = memory[ip];

        match opcode {
            ADD => {
                let a = memory[memory[ip + 1] as usize];
                let b = memory[memory[ip + 2] as usize];
                let t = memory[ip + 3] as usize;

                memory[t] = a + b;
            }
            MULTIPLY => {
                let a = memory[memory[ip + 1] as usize];
                let b = memory[memory[ip + 2] as usize];
                let t = memory[ip + 3] as usize;

                memory[t] = a * b;
            }
            HALT => break,
            _ => {}
        }

        ip += 4;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addition() {
        let mut memory = vec![1, 0, 0, 0, 99];

        run(&mut memory);

        assert_eq!(vec![2, 0, 0, 0, 99], memory);
    }

    #[test]
    fn multiplication() {
        let mut memory = vec![2, 3, 0, 3, 99];

        run(&mut memory);

        assert_eq!(vec![2, 3, 0, 6, 99], memory);
    }

    #[test]
    fn large_multiplication() {
        let mut memory = vec![2, 4, 4, 5, 99, 0];

        run(&mut memory);

        assert_eq!(vec![2, 4, 4, 5, 99, 9801], memory);
    }

    #[test]
    fn multiple_operations() {
        let mut memory = vec![1, 1, 1, 4, 99, 5, 6, 0, 99];

        run(&mut memory);

        assert_eq!(vec![30, 1, 1, 4, 2, 5, 6, 0, 99], memory);
    }
}
