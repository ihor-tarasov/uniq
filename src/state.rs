use crate::{VMRes, VMError, opcode};

fn fetch_u8(opcodes: &[u8], offset: usize) -> VMRes<u8> {
    match opcodes.get(offset) {
        Some(data) => Ok(*data),
        None => Err(VMError::OpcodeFetch),
    }
}

fn fetch_u16(opcodes: &[u8], offset: usize) -> VMRes<u16> {
    Ok(u16::from_be_bytes([
        fetch_u8(opcodes, offset + 0)?,
        fetch_u8(opcodes, offset + 1)?,
    ]))
}

fn fetch_u64(opcodes: &[u8], offset: usize) -> VMRes<u64> {
    Ok(u64::from_be_bytes([
        fetch_u8(opcodes, offset + 0)?,
        fetch_u8(opcodes, offset + 1)?,
        fetch_u8(opcodes, offset + 2)?,
        fetch_u8(opcodes, offset + 3)?,
        fetch_u8(opcodes, offset + 4)?,
        fetch_u8(opcodes, offset + 5)?,
        fetch_u8(opcodes, offset + 6)?,
        fetch_u8(opcodes, offset + 7)?,
    ]))
}

pub struct State<'a> {
    stack: &'a mut [i64],
    stack_pointer: usize,
    program_counter: usize,
}

impl<'a> State<'a> {
    pub fn new(stack: &'a mut [i64]) -> Self {
        Self {
            stack,
            stack_pointer: 0,
            program_counter: 0,
        }
    }

    pub fn push(&mut self, value: i64) -> VMRes {
        if self.stack_pointer < self.stack.len() {
            self.stack[self.stack_pointer] = value;
            self.stack_pointer += 1;
            Ok(())
        } else {
            Err(VMError::StackOverflow)
        }
    }

    pub fn pop(&mut self) -> VMRes<i64> {
        if self.stack_pointer > self.stack.len() {
            Err(VMError::StackOverflow)
        } else if self.stack_pointer == 0 {
            Err(VMError::StackUnderflow)
        } else {
            self.stack_pointer -= 1;
            Ok(self.stack[self.stack_pointer])
        }
    }

    fn ret(&mut self) -> VMRes<bool> {
        Ok(false)
    }

    fn add(&mut self, l: i64, r: i64) -> VMRes<i64> {
        Ok(l.wrapping_add(r))
    }

    fn sub(&mut self, l: i64, r: i64) -> VMRes<i64> {
        Ok(l.wrapping_sub(r))
    }

    fn mul(&mut self, l: i64, r: i64) -> VMRes<i64> {
        Ok(l.wrapping_mul(r))
    }

    fn bin<F>(&mut self, f: F) -> VMRes<bool>
    where
        F: Fn(&mut Self, i64, i64) -> VMRes<i64>,
    {
        let r = self.pop()?;
        let l = self.pop()?;
        let res = f(self, l, r)?;
        self.push(res)?;
        self.program_counter += 1;
        Ok(true)
    }

    fn int1(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        let value = fetch_u8(opcodes, self.program_counter + 1)?;
        self.push(value as i64)?;
        self.program_counter += 2;
        Ok(true)
    }

    fn int2(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        let value = fetch_u16(opcodes, self.program_counter + 1)?;
        self.push(value as i64)?;
        self.program_counter += 3;
        Ok(true)
    }

    fn int8(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        let value = fetch_u64(opcodes, self.program_counter + 1)?;
        self.push(value as i64)?;
        self.program_counter += 9;
        Ok(true)
    }

    pub fn step(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        let opcode = fetch_u8(opcodes, self.program_counter)?;
        match opcode {
            opcode::RET => self.ret(),
            opcode::INT1 => self.int1(opcodes),
            opcode::INT2 => self.int2(opcodes),
            opcode::INT8 => self.int8(opcodes),
            opcode::ADD => self.bin(Self::add),
            opcode::SUB => self.bin(Self::sub),
            opcode::MUL => self.bin(Self::mul),
            _ => Err(VMError::UnknownOpcode),
        }
    }

    pub fn run(&mut self, opcodes: &[u8]) -> VMRes<i64> {
        while self.step(opcodes)? {}
        self.pop()
    }
}
