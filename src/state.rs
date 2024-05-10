use crate::{vm_error, Instruction, Program, VMResult, Value};

const STACK_SIZE: usize = 256;

pub struct State {
    stack: [Value; STACK_SIZE],
    stack_pointer: usize,
    program_counter: usize,
}

impl State {
    pub fn new() -> Self {
        Self {
            stack: [Value::Void; STACK_SIZE],
            stack_pointer: 0,
            program_counter: 0,
        }
    }

    fn push(&mut self, value: Value) -> VMResult {
        if self.stack_pointer < STACK_SIZE {
            self.stack[self.stack_pointer] = value;
            self.stack_pointer += 1;
            Ok(())
        } else {
            vm_error(format!("Stack overflow"))
        }
    }

    fn pop(&mut self) -> VMResult<Value> {
        if self.stack_pointer == 0 {
            vm_error(format!("Stack underflow"))
        } else {
            self.stack_pointer -= 1;
            Ok(self.stack[self.stack_pointer])
        }
    }

    fn integer(&mut self, value: i64) -> VMResult<bool> {
        self.push(Value::Integer(value))?;
        self.program_counter += 1;
        Ok(true)
    }

    fn float(&mut self, value: f64) -> VMResult<bool> {
        self.push(Value::Float(value))?;
        self.program_counter += 1;
        Ok(true)
    }

    fn binary<F>(&mut self, f: F) -> VMResult<bool>
    where
        F: Fn(&mut Self, Value, Value) -> VMResult<Value>,
    {
        let right = self.pop()?;
        let left = self.pop()?;
        let result = f(self, left, right)?;
        self.push(result)?;
        self.program_counter += 1;
        Ok(true)
    }

    fn end(&mut self) -> VMResult<bool> {
        Ok(false)
    }

    fn fetch(&mut self, program: &Program) -> VMResult<Instruction> {
        match program.instruction(self.program_counter) {
            Some(instruction) => Ok(instruction),
            None => vm_error(format!("Program counter out of bounds")),
        }
    }

    fn step(&mut self, program: &Program) -> VMResult<bool> {
        let instruction = self.fetch(program)?;
        match instruction {
            Instruction::Integer(value) => self.integer(value),
            Instruction::Float(value) => self.float(value),
            Instruction::Addict => self.binary(Self::addict),
            Instruction::Subtract => self.binary(Self::subtract),
            Instruction::Multiply => self.binary(Self::multiply),
            Instruction::Divide => self.binary(Self::divide),
            Instruction::Modulo => self.binary(Self::modulo),
            Instruction::Equals => self.binary(Self::equals),
            Instruction::NotEquals => self.binary(Self::not_equals),
            Instruction::Less => self.binary(Self::less),
            Instruction::Greater => self.binary(Self::greater),
            Instruction::LessEquals => self.binary(Self::less_equals),
            Instruction::GreaterEquals => self.binary(Self::greater_equals),
            Instruction::End => self.end(),
        }
    }

    pub fn run(&mut self, program: &Program) -> VMResult<Value> {
        while self.step(program)? {}
        self.pop()
    }

    pub fn program_counter(&self) -> usize {
        self.program_counter
    }
}
