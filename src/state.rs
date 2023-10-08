use crate::{VMRes, VMError, opcode, Value};

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

fn fetch_f64(opcodes: &[u8], offset: usize) -> VMRes<f64> {
    Ok(f64::from_be_bytes([
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
    stack: &'a mut [Value],
    stack_pointer: usize,
    program_counter: usize,
    message: Option<String>,
}

impl<'a> State<'a> {
    pub fn new(stack: &'a mut [Value]) -> Self {
        Self {
            stack,
            stack_pointer: 0,
            program_counter: 0,
            message: None,
        }
    }

    pub fn push(&mut self, value: Value) -> VMRes {
        if self.stack_pointer < self.stack.len() {
            self.stack[self.stack_pointer] = value;
            self.stack_pointer += 1;
            Ok(())
        } else {
            Err(VMError::StackOverflow)
        }
    }

    pub fn pop(&mut self) -> VMRes<Value> {
        if self.stack_pointer > self.stack.len() {
            Err(VMError::StackOverflow)
        } else if self.stack_pointer == 0 {
            Err(VMError::StackUnderflow)
        } else {
            self.stack_pointer -= 1;
            Ok(self.stack[self.stack_pointer].clone())
        }
    }

    fn ret(&mut self) -> VMRes<bool> {
        Ok(false)
    }

    fn add(&mut self, l: Value, r: Value) -> VMRes<Value> {
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l.wrapping_add(r))),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Real((l as f64) + r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l + (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l + r)),
            _ => {
                self.message = Some(format!("Unable to addict {l} and {r} values."));
                Err(VMError::BinaryOperation)
            }
        }
    }

    fn sub(&mut self, l: Value, r: Value) -> VMRes<Value> {
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l.wrapping_sub(r))),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Real((l as f64) - r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l - (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l - r)),
            _ => {
                self.message = Some(format!("Unable to subtract {l} and {r} values."));
                Err(VMError::BinaryOperation)
            }
        }
    }

    fn mul(&mut self, l: Value, r: Value) -> VMRes<Value> {
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l.wrapping_mul(r))),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Real((l as f64) * r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l * (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l * r)),
            _ => {
                self.message = Some(format!("Unable to multiply {l} and {r} values."));
                Err(VMError::BinaryOperation)
            }
        }
    }

    fn eq(&mut self, l: Value, r: Value) -> VMRes<Value> {
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l == r)),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((l as f64) == r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(l == (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l == r)),
            _ => {
                self.message = Some(format!("Unable to compare {l} and {r} values."));
                Err(VMError::BinaryOperation)
            }
        }
    }

    fn bin<F>(&mut self, f: F) -> VMRes<bool>
    where
        F: Fn(&mut Self, Value, Value) -> VMRes<Value>,
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
        self.push(Value::Integer(value as i64))?;
        self.program_counter += 2;
        Ok(true)
    }

    fn int2(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        let value = fetch_u16(opcodes, self.program_counter + 1)?;
        self.push(Value::Integer(value as i64))?;
        self.program_counter += 3;
        Ok(true)
    }

    fn int8(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        let value = fetch_u64(opcodes, self.program_counter + 1)?;
        self.push(Value::Integer(value as i64))?;
        self.program_counter += 9;
        Ok(true)
    }

    fn real(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        let value = fetch_f64(opcodes, self.program_counter + 1)?;
        self.push(Value::Real(value))?;
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
            opcode::REAL => self.real(opcodes),
            opcode::ADD => self.bin(Self::add),
            opcode::SUB => self.bin(Self::sub),
            opcode::MUL => self.bin(Self::mul),
            opcode::EQ => self.bin(Self::eq),
            _ => Err(VMError::UnknownOpcode),
        }
    }

    pub fn run(&mut self, opcodes: &[u8]) -> VMRes<Value> {
        while self.step(opcodes)? {}
        self.pop()
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_ref().map(|s| s.as_str())
    }
}
