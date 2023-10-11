use std::{cell::RefCell, rc::Rc};

use crate::{opcode, Object, VMError, VMRes, Value};

const DUMP_STACK: bool = true;
const DUMP_OPCODE: bool = true;
const DUMP_OPCODES: bool = true;

macro_rules! dumpln {
    () => {
        if DUMP_OPCODE {
            println!();
        }
    };
    ($($arg:tt)*) => {{
        if DUMP_OPCODE {
            println!($($arg)*);
        }
    }};
}

fn checked_add(a: u32, b: u32) -> VMRes<u32> {
    a.checked_add(b).ok_or(VMError::AddressOverflow)
}

fn checked_as(a: usize) -> VMRes<u32> {
    if a <= u32::MAX as usize {
        Ok(a as u32)
    } else {
        Err(VMError::AddressOverflow)
    }
}

fn fetch_u8(opcodes: &[u8], offset: u32) -> VMRes<u8> {
    match opcodes.get(offset as usize) {
        Some(data) => Ok(*data),
        None => Err(VMError::OpcodeFetch),
    }
}

fn fetch_u16(opcodes: &[u8], offset: u32) -> VMRes<u16> {
    Ok(u16::from_be_bytes([
        fetch_u8(opcodes, offset)?,
        fetch_u8(opcodes, checked_add(offset, 1)?)?,
    ]))
}

fn fetch_u32(opcodes: &[u8], offset: u32) -> VMRes<u32> {
    Ok(u32::from_be_bytes([
        fetch_u8(opcodes, offset)?,
        fetch_u8(opcodes, checked_add(offset, 1)?)?,
        fetch_u8(opcodes, checked_add(offset, 2)?)?,
        fetch_u8(opcodes, checked_add(offset, 3)?)?,
    ]))
}

fn fetch_u64(opcodes: &[u8], offset: u32) -> VMRes<u64> {
    Ok(u64::from_be_bytes([
        fetch_u8(opcodes, offset)?,
        fetch_u8(opcodes, checked_add(offset, 1)?)?,
        fetch_u8(opcodes, checked_add(offset, 2)?)?,
        fetch_u8(opcodes, checked_add(offset, 3)?)?,
        fetch_u8(opcodes, checked_add(offset, 4)?)?,
        fetch_u8(opcodes, checked_add(offset, 5)?)?,
        fetch_u8(opcodes, checked_add(offset, 6)?)?,
        fetch_u8(opcodes, checked_add(offset, 7)?)?,
    ]))
}

fn fetch_f64(opcodes: &[u8], offset: u32) -> VMRes<f64> {
    Ok(f64::from_be_bytes([
        fetch_u8(opcodes, offset)?,
        fetch_u8(opcodes, checked_add(offset, 1)?)?,
        fetch_u8(opcodes, checked_add(offset, 2)?)?,
        fetch_u8(opcodes, checked_add(offset, 3)?)?,
        fetch_u8(opcodes, checked_add(offset, 4)?)?,
        fetch_u8(opcodes, checked_add(offset, 5)?)?,
        fetch_u8(opcodes, checked_add(offset, 6)?)?,
        fetch_u8(opcodes, checked_add(offset, 7)?)?,
    ]))
}

fn dump_opcodes(opcodes: &[u8]) -> VMRes {
    println!("# Stack size: {}", fetch_u32(opcodes, 0)?);
    let mut i = 4;
    while i < checked_as(opcodes.len())? {
        print!("{i}|");
        let opcode = fetch_u8(opcodes, i)?;
        i = checked_add(i, 1)?;
        match opcode {
            opcode::RET => println!("RET"),
            opcode::INT1 => {
                let value = fetch_u8(opcodes, i)?;
                println!("INT {value}");
                i = checked_add(i, 1)?;
            }
            opcode::INT2 => {
                let value = fetch_u16(opcodes, i)?;
                println!("INT {value}");
                i = checked_add(i, 2)?;
            }
            opcode::INT8 => {
                let value = fetch_u64(opcodes, i)?;
                println!("INT {value}");
                i = checked_add(i, 8)?;
            }
            opcode::TRUE => println!("TRUE"),
            opcode::FALSE => println!("FALSE"),
            opcode::REAL => {
                let value = fetch_f64(opcodes, i)?;
                println!("REAL {value}");
                i = checked_add(i, 8)?;
            }
            opcode::ADD => println!("ADD"),
            opcode::SUB => println!("SUB"),
            opcode::MUL => println!("MUL"),
            opcode::DIV => println!("DIV"),
            opcode::EQ => println!("EQ"),
            opcode::NE => println!("NE"),
            opcode::LS => println!("LS"),
            opcode::GR => println!("GR"),
            opcode::LE => println!("LE"),
            opcode::GE => println!("GE"),
            opcode::JP2 => {
                let value = fetch_u16(opcodes, i)?;
                println!("JP {value}");
                i = checked_add(i, 4)?;
            }
            opcode::JP4 => {
                let value = fetch_u32(opcodes, i)?;
                println!("JP {value}");
                i = checked_add(i, 4)?;
            }
            opcode::JF2 => {
                let value = fetch_u16(opcodes, i)?;
                println!("JF {value}");
                i = checked_add(i, 4)?;
            }
            opcode::JF4 => {
                let value = fetch_u32(opcodes, i)?;
                println!("JF {value}");
                i = checked_add(i, 4)?;
            }
            opcode::DROP => println!("DROP"),
            opcode::VOID => println!("VOID"),
            opcode::LIST => println!("LIST"),
            opcode::CALL => {
                let value = fetch_u8(opcodes, i)?;
                println!("CALL {value}");
                i = checked_add(i, 1)?;
            }
            opcode::GET => println!("GET"),
            opcode::SET => println!("SET"),
            opcode::LD1 => {
                let value = fetch_u8(opcodes, i)?;
                println!("LD {value}");
                i = checked_add(i, 1)?;
            }
            opcode::LD2 => {
                let value = fetch_u16(opcodes, i)?;
                println!("LD {value}");
                i = checked_add(i, 2)?;
            }
            opcode::ST1 => {
                let value = fetch_u8(opcodes, i)?;
                println!("ST {value}");
                i = checked_add(i, 1)?;
            }
            opcode::ST2 => {
                let value = fetch_u16(opcodes, i)?;
                println!("ST {value}");
                i = checked_add(i, 2)?;
            }
            _ => return Err(VMError::UnknownOpcode),
        }
    }
    Ok(())
}

pub struct State<'a> {
    stack: &'a mut [Value],
    stack_pointer: u32,
    program_counter: u32,
    locals: u32,
    message: Option<String>,
}

impl<'a> State<'a> {
    pub fn new(stack: &'a mut [Value]) -> Self {
        assert!(
            stack.len() <= u32::MAX as usize,
            "Maximum stack length must be u32::MAX."
        );
        Self {
            stack,
            stack_pointer: 0,
            program_counter: 0,
            locals: 0,
            message: None,
        }
    }

    pub fn push(&mut self, value: Value) -> VMRes {
        if self.stack_pointer < self.stack.len() as u32 {
            self.stack[self.stack_pointer as usize] = value;
            self.stack_pointer += 1;
            Ok(())
        } else {
            Err(VMError::StackOverflow)
        }
    }

    pub fn pop(&mut self) -> VMRes<Value> {
        if self.stack_pointer >= self.stack.len() as u32 {
            Err(VMError::StackOverflow)
        } else if self.stack_pointer == 0 {
            Err(VMError::StackUnderflow)
        } else {
            self.stack_pointer -= 1;
            Ok(self.stack[self.stack_pointer as usize].clone())
        }
    }

    fn peek(&mut self) -> VMRes<Value> {
        if self.stack_pointer >= self.stack.len() as u32 {
            Err(VMError::StackOverflow)
        } else if self.stack_pointer == 0 {
            Err(VMError::StackUnderflow)
        } else {
            Ok(self.stack[(self.stack_pointer - 1) as usize].clone())
        }
    }

    fn dump_stack(&self) {
        for i in 0..self.stack_pointer {
            print!("[{}]", self.stack[i as usize]);
        }
        println!();
    }

    fn ret(&mut self) -> VMRes<bool> {
        dumpln!("RET");
        Ok(false)
    }

    fn add(&mut self, l: Value, r: Value) -> VMRes<Value> {
        dumpln!("ADD");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l.wrapping_add(r))),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Real((l as f64) + r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l + (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l + r)),
            (Value::Object(object), value) => {
                {
                    let mut object = object.borrow_mut();
                    object.push(value);
                }
                Ok(Value::Object(object))
            }
            _ => {
                self.message = Some(format!("Unable to addict {l} and {r} values."));
                Err(VMError::BinaryOperation)
            }
        }
    }

    fn sub(&mut self, l: Value, r: Value) -> VMRes<Value> {
        dumpln!("SUB");
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
        dumpln!("MUL");
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

    fn div(&mut self, l: Value, r: Value) -> VMRes<Value> {
        dumpln!("DIV");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => {
                if r == 0 {
                    Err(VMError::DividingByZero)
                } else {
                    Ok(Value::Integer(l.wrapping_div(r)))
                }
            }
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Real((l as f64) / r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l / (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l / r)),
            _ => {
                self.message = Some(format!("Unable to divide {l} and {r} values."));
                Err(VMError::BinaryOperation)
            }
        }
    }

    fn eq(&mut self, l: Value, r: Value) -> VMRes<Value> {
        dumpln!("EQ");
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

    fn ne(&mut self, l: Value, r: Value) -> VMRes<Value> {
        dumpln!("NE");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l != r)),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((l as f64) != r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(l != (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l != r)),
            _ => {
                self.message = Some(format!("Unable to compare {l} and {r} values."));
                Err(VMError::BinaryOperation)
            }
        }
    }

    fn ls(&mut self, l: Value, r: Value) -> VMRes<Value> {
        dumpln!("LS");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l < r)),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((l as f64) < r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(l < (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l < r)),
            _ => {
                self.message = Some(format!("Unable to compare {l} and {r} values."));
                Err(VMError::BinaryOperation)
            }
        }
    }

    fn le(&mut self, l: Value, r: Value) -> VMRes<Value> {
        dumpln!("LE");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l <= r)),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((l as f64) <= r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(l <= (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l <= r)),
            _ => {
                self.message = Some(format!("Unable to compare {l} and {r} values."));
                Err(VMError::BinaryOperation)
            }
        }
    }

    fn gr(&mut self, l: Value, r: Value) -> VMRes<Value> {
        dumpln!("GR");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l > r)),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((l as f64) > r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(l > (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l > r)),
            _ => {
                self.message = Some(format!("Unable to compare {l} and {r} values."));
                Err(VMError::BinaryOperation)
            }
        }
    }

    fn ge(&mut self, l: Value, r: Value) -> VMRes<Value> {
        dumpln!("GE");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l >= r)),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((l as f64) >= r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(l >= (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l >= r)),
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
        self.program_counter = checked_add(self.program_counter, 1)?;
        Ok(true)
    }

    fn int1(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        let value = fetch_u8(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("INT {value}");
        self.push(Value::Integer(value as i64))?;
        self.program_counter = checked_add(self.program_counter, 2)?;
        Ok(true)
    }

    fn int2(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        let value = fetch_u16(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("INT {value}");
        self.push(Value::Integer(value as i64))?;
        self.program_counter = checked_add(self.program_counter, 3)?;
        Ok(true)
    }

    fn int8(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        let value = fetch_u64(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("INT {value}");
        self.push(Value::Integer(value as i64))?;
        self.program_counter = checked_add(self.program_counter, 9)?;
        Ok(true)
    }

    fn jp2(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        self.program_counter = fetch_u16(opcodes, checked_add(self.program_counter, 1)?)? as u32;
        dumpln!("JP {}", self.program_counter);
        Ok(true)
    }

    fn jp4(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        self.program_counter = fetch_u32(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("JP {}", self.program_counter);
        Ok(true)
    }

    fn jf2(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        dumpln!(
            "JF {}",
            fetch_u16(opcodes, checked_add(self.program_counter, 1)?)?
        );
        let value = self.pop()?;
        match value {
            Value::Boolean(value) => {
                if value {
                    self.program_counter = checked_add(self.program_counter, 5)?;
                } else {
                    self.program_counter =
                        fetch_u16(opcodes, checked_add(self.program_counter, 1)?)? as u32;
                }
                Ok(true)
            }
            _ => {
                self.message = Some(format!("Expected bool value, found {value}"));
                Err(VMError::UnexpectedType)
            }
        }
    }

    fn jf4(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        dumpln!(
            "JF {}",
            fetch_u32(opcodes, checked_add(self.program_counter, 1)?)?
        );
        let value = self.pop()?;
        match value {
            Value::Boolean(value) => {
                if value {
                    self.program_counter = checked_add(self.program_counter, 5)?;
                } else {
                    self.program_counter =
                        fetch_u32(opcodes, checked_add(self.program_counter, 1)?)?;
                }
                Ok(true)
            }
            _ => {
                self.message = Some(format!("Expected bool value, found {value}"));
                Err(VMError::UnexpectedType)
            }
        }
    }

    fn real(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        let value = fetch_f64(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("REAL {value}");
        self.push(Value::Real(value))?;
        self.program_counter = checked_add(self.program_counter, 9)?;
        Ok(true)
    }

    fn boolean(&mut self, value: bool) -> VMRes<bool> {
        if value {
            dumpln!("TRUE");
        } else {
            dumpln!("FALSE");
        }
        self.push(Value::Boolean(value))?;
        self.program_counter = checked_add(self.program_counter, 1)?;
        Ok(true)
    }

    fn drop(&mut self) -> VMRes<bool> {
        dumpln!("DROP");
        self.pop()?;
        self.program_counter = checked_add(self.program_counter, 1)?;
        Ok(true)
    }

    fn void(&mut self) -> VMRes<bool> {
        dumpln!("VOID");
        self.push(Value::Void)?;
        self.program_counter = checked_add(self.program_counter, 1)?;
        Ok(true)
    }

    fn list(&mut self) -> VMRes<bool> {
        dumpln!("LIST");
        self.push(Value::Object(Rc::new(RefCell::new(Object::List(
            Vec::new(),
        )))))?;
        self.program_counter = checked_add(self.program_counter, 1)?;
        Ok(true)
    }

    fn ld1(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        let index = fetch_u8(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("LD {index}");
        if self.locals + index as u32 >= self.stack.len() as u32 {
            return Err(VMError::StackOverflow);
        }
        self.push(self.stack[(self.locals + index as u32) as usize].clone())?;
        self.program_counter = checked_add(self.program_counter, 2)?;
        Ok(true)
    }

    fn ld2(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        let index = fetch_u16(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("LD {index}");
        if self.locals + index as u32 >= self.stack.len() as u32 {
            return Err(VMError::StackOverflow);
        }
        self.push(self.stack[(self.locals + index as u32) as usize].clone())?;
        self.program_counter = checked_add(self.program_counter, 3)?;
        Ok(true)
    }

    fn ld4(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        let index = fetch_u32(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("LD {index}");
        if self.locals + index >= self.stack.len() as u32 {
            return Err(VMError::StackOverflow);
        }
        self.push(self.stack[(self.locals + index) as usize].clone())?;
        self.program_counter = checked_add(self.program_counter, 5)?;
        Ok(true)
    }

    fn st1(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        let index = fetch_u8(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("ST {index}");
        if self.locals + index as u32 >= self.stack.len() as u32 {
            return Err(VMError::StackOverflow);
        }
        self.stack[(self.locals + index as u32) as usize] = self.peek()?;
        self.program_counter = checked_add(self.program_counter, 2)?;
        Ok(true)
    }

    fn st2(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        let index = fetch_u16(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("ST {index}");
        if self.locals + index as u32 >= self.stack.len() as u32 {
            return Err(VMError::StackOverflow);
        }
        self.stack[(self.locals + index as u32) as usize] = self.peek()?;
        self.program_counter = checked_add(self.program_counter, 3)?;
        Ok(true)
    }

    fn st4(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        let index = fetch_u32(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("ST {index}");
        if self.locals + index >= self.stack.len() as u32 {
            return Err(VMError::StackOverflow);
        }
        self.stack[(self.locals + index) as usize] = self.peek()?;
        self.program_counter = checked_add(self.program_counter, 5)?;
        Ok(true)
    }

    pub fn step(&mut self, opcodes: &[u8]) -> VMRes<bool> {
        let opcode = fetch_u8(opcodes, self.program_counter)?;
        match opcode {
            opcode::RET => self.ret(),
            opcode::INT1 => self.int1(opcodes),
            opcode::INT2 => self.int2(opcodes),
            opcode::INT8 => self.int8(opcodes),
            opcode::TRUE => self.boolean(true),
            opcode::FALSE => self.boolean(false),
            opcode::REAL => self.real(opcodes),
            opcode::ADD => self.bin(Self::add),
            opcode::SUB => self.bin(Self::sub),
            opcode::MUL => self.bin(Self::mul),
            opcode::DIV => self.bin(Self::div),
            opcode::EQ => self.bin(Self::eq),
            opcode::NE => self.bin(Self::ne),
            opcode::LS => self.bin(Self::ls),
            opcode::GR => self.bin(Self::gr),
            opcode::LE => self.bin(Self::le),
            opcode::GE => self.bin(Self::ge),
            opcode::JP2 => self.jp2(opcodes),
            opcode::JP4 => self.jp4(opcodes),
            opcode::JF2 => self.jf2(opcodes),
            opcode::JF4 => self.jf4(opcodes),
            opcode::DROP => self.drop(),
            opcode::VOID => self.void(),
            opcode::LIST => self.list(),
            opcode::LD1 => self.ld1(opcodes),
            opcode::LD2 => self.ld2(opcodes),
            opcode::LD4 => self.ld4(opcodes),
            opcode::ST1 => self.st1(opcodes),
            opcode::ST2 => self.st2(opcodes),
            opcode::ST4 => self.st4(opcodes),
            _ => Err(VMError::UnknownOpcode),
        }
    }

    pub fn run(&mut self, opcodes: &[u8]) -> VMRes<Value> {
        if DUMP_OPCODES {
            println!("# OPCODES DUMP");
            dump_opcodes(opcodes)?;
        }
        if DUMP_OPCODE {
            println!("# RUNTIME DUMP");
        }

        let stack_size = fetch_u32(opcodes, 0)?;
        self.stack_pointer = stack_size;
        self.program_counter = checked_add(self.program_counter, 4)?;

        while self.step(opcodes)? {
            if DUMP_STACK {
                self.dump_stack();
            }
        }
        self.pop()
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_ref().map(|s| s.as_str())
    }

    pub fn program_counter(&self) -> u32 {
        self.program_counter
    }
}
