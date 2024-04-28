use crate::{Instruction, Program, VMError};

const STACK_SIZE: usize = 256;

pub struct State {
    stack: [i32; STACK_SIZE],
    stack_pointer: usize,
    program_counter: usize,
}

impl State {
    pub fn new() -> Self {
        Self {
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
            program_counter: 0,
        }
    }

    fn push(&mut self, value: i32) -> Result<(), VMError> {
        if self.stack_pointer < STACK_SIZE {
            self.stack[self.stack_pointer] = value;
            self.stack_pointer += 1;
            Ok(())
        } else {
            Err(VMError::StackOverflow)
        }
    }

    fn pop(&mut self) -> Result<i32, VMError> {
        if self.stack_pointer == 0 {
            Err(VMError::StackUnderflow)
        } else {
            self.stack_pointer -= 1;
            Ok(self.stack[self.stack_pointer])
        }
    }

    fn integer(&mut self, value: i32) -> Result<bool, VMError> {
        self.push(value)?;
        self.program_counter += 1;
        Ok(true)
    }

    fn binary<F>(&mut self, f: F) -> Result<bool, VMError>
    where
        F: Fn(&mut Self, i32, i32) -> Result<i32, VMError>
    {
        let right = self.pop()?;
        let left = self.pop()?;
        let result = f(self, left, right)?;
        self.push(result)?;
        self.program_counter += 1;
        Ok(true)
    }

    fn addict(&mut self, left: i32, right: i32) -> Result<i32, VMError> {
        Ok(left.wrapping_add(right))
    }

    fn subtract(&mut self, left: i32, right: i32) -> Result<i32, VMError> {
        Ok(left.wrapping_sub(right))
    }

    fn multiply(&mut self, left: i32, right: i32) -> Result<i32, VMError> {
        Ok(left.wrapping_mul(right))
    }

    fn divide(&mut self, left: i32, right: i32) -> Result<i32, VMError> {
        if right == 0 {
            Err(VMError::DividingByZero)
        } else {
            Ok(left.wrapping_div(right))
        }
    }

    fn end(&mut self) -> Result<bool, VMError> {
        Ok(false)
    }

    fn fetch(&mut self, program: &Program) -> Result<Instruction, VMError> {
        program
            .instruction(self.program_counter)
            .ok_or(VMError::ProgramCounterOutOfBounds)
    }

    fn step(&mut self, program: &Program) -> Result<bool, VMError> {
        let instruction = self.fetch(program)?;
        match instruction {
            Instruction::Integer(value) => self.integer(value),
            Instruction::Addict => self.binary(Self::addict),
            Instruction::Subtract => self.binary(Self::subtract),
            Instruction::Multiply => self.binary(Self::multiply),
            Instruction::Divide => self.binary(Self::divide),
            Instruction::End => self.end(),
        }
    }

    pub fn run(&mut self, program: &Program) -> Result<i32, VMError> {
        while self.step(program)? {}
        self.pop()
    }
}
