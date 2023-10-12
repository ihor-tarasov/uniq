use std::{cell::RefCell, rc::Rc};

use crate::{opcode, vm::Object};

use super::{Res, Error, Value};

const DUMP_STACK: bool = false;
const DUMP_OPCODE: bool = false;
const DUMP_OPCODES: bool = false;

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

fn checked_add(a: u32, b: u32) -> Res<u32> {
    a.checked_add(b).ok_or(Error::AddressOverflow)
}

fn checked_as(a: usize) -> Res<u32> {
    if a <= u32::MAX as usize {
        Ok(a as u32)
    } else {
        Err(Error::AddressOverflow)
    }
}

fn fetch_u8(opcodes: &[u8], offset: u32) -> Res<u8> {
    match opcodes.get(offset as usize) {
        Some(data) => Ok(*data),
        None => Err(Error::OpcodeFetch),
    }
}

fn fetch_u16(opcodes: &[u8], offset: u32) -> Res<u16> {
    Ok(u16::from_be_bytes([
        fetch_u8(opcodes, offset)?,
        fetch_u8(opcodes, checked_add(offset, 1)?)?,
    ]))
}

fn fetch_u32(opcodes: &[u8], offset: u32) -> Res<u32> {
    Ok(u32::from_be_bytes([
        fetch_u8(opcodes, offset)?,
        fetch_u8(opcodes, checked_add(offset, 1)?)?,
        fetch_u8(opcodes, checked_add(offset, 2)?)?,
        fetch_u8(opcodes, checked_add(offset, 3)?)?,
    ]))
}

fn fetch_u64(opcodes: &[u8], offset: u32) -> Res<u64> {
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

fn fetch_f64(opcodes: &[u8], offset: u32) -> Res<f64> {
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

fn dump_opcodes(opcodes: &[u8]) -> Res {
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
            opcode::PTR => {
                let value = fetch_u32(opcodes, i)?;
                println!("PTR {value}");
                i = checked_add(i, 4)?;
            }
            _ => return Err(Error::UnknownOpcode),
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

    pub fn push(&mut self, value: Value) -> Res {
        if self.stack_pointer < self.stack.len() as u32 {
            self.stack[self.stack_pointer as usize] = value;
            self.stack_pointer += 1;
            Ok(())
        } else {
            Err(Error::StackOverflow)
        }
    }

    pub fn pop(&mut self) -> Res<Value> {
        if self.stack_pointer >= self.stack.len() as u32 {
            Err(Error::StackOverflow)
        } else if self.stack_pointer == 0 {
            Err(Error::StackUnderflow)
        } else {
            self.stack_pointer -= 1;
            Ok(self.stack[self.stack_pointer as usize].clone())
        }
    }

    fn peek(&mut self) -> Res<Value> {
        if self.stack_pointer >= self.stack.len() as u32 {
            Err(Error::StackOverflow)
        } else if self.stack_pointer == 0 {
            Err(Error::StackUnderflow)
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

    fn ret(&mut self) -> Res<bool> {
        dumpln!("RET");
        if self.locals == 0 {
            return Ok(false);
        }
        let result = self.pop()?;
        self.stack_pointer = self.locals - 1;
        let call_state = self.stack[self.stack_pointer as usize].clone();
        match call_state {
            Value::CallState(new_pc, new_locals) => {
                self.push(result)?;
                self.program_counter = new_pc;
                self.locals = new_locals;
            }
            _ => return self.error(format!("Expected CallState, found {call_state}")),
        }
        Ok(true)
    }

    fn add(&mut self, l: Value, r: Value) -> Res<Value> {
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
                Err(Error::BinaryOperation)
            }
        }
    }

    fn sub(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("SUB");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l.wrapping_sub(r))),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Real((l as f64) - r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l - (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l - r)),
            _ => {
                self.message = Some(format!("Unable to subtract {l} and {r} values."));
                Err(Error::BinaryOperation)
            }
        }
    }

    fn mul(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("MUL");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l.wrapping_mul(r))),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Real((l as f64) * r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l * (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l * r)),
            _ => {
                self.message = Some(format!("Unable to multiply {l} and {r} values."));
                Err(Error::BinaryOperation)
            }
        }
    }

    fn div(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("DIV");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => {
                if r == 0 {
                    Err(Error::DividingByZero)
                } else {
                    Ok(Value::Integer(l.wrapping_div(r)))
                }
            }
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Real((l as f64) / r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l / (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l / r)),
            _ => {
                self.message = Some(format!("Unable to divide {l} and {r} values."));
                Err(Error::BinaryOperation)
            }
        }
    }

    fn eq(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("EQ");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l == r)),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((l as f64) == r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(l == (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l == r)),
            _ => {
                self.message = Some(format!("Unable to compare {l} and {r} values."));
                Err(Error::BinaryOperation)
            }
        }
    }

    fn ne(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("NE");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l != r)),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((l as f64) != r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(l != (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l != r)),
            _ => {
                self.message = Some(format!("Unable to compare {l} and {r} values."));
                Err(Error::BinaryOperation)
            }
        }
    }

    fn ls(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("LS");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l < r)),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((l as f64) < r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(l < (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l < r)),
            _ => {
                self.message = Some(format!("Unable to compare {l} and {r} values."));
                Err(Error::BinaryOperation)
            }
        }
    }

    fn le(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("LE");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l <= r)),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((l as f64) <= r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(l <= (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l <= r)),
            _ => {
                self.message = Some(format!("Unable to compare {l} and {r} values."));
                Err(Error::BinaryOperation)
            }
        }
    }

    fn gr(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("GR");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l > r)),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((l as f64) > r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(l > (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l > r)),
            _ => {
                self.message = Some(format!("Unable to compare {l} and {r} values."));
                Err(Error::BinaryOperation)
            }
        }
    }

    fn ge(&mut self, l: Value, r: Value) -> Res<Value> {
        dumpln!("GE");
        match (l.clone(), r.clone()) {
            (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l >= r)),
            (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((l as f64) >= r)),
            (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(l >= (r as f64))),
            (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l >= r)),
            _ => {
                self.message = Some(format!("Unable to compare {l} and {r} values."));
                Err(Error::BinaryOperation)
            }
        }
    }

    fn bin<F>(&mut self, f: F) -> Res<bool>
    where
        F: Fn(&mut Self, Value, Value) -> Res<Value>,
    {
        let r = self.pop()?;
        let l = self.pop()?;
        let res = f(self, l, r)?;
        self.push(res)?;
        self.program_counter = checked_add(self.program_counter, 1)?;
        Ok(true)
    }

    fn int1(&mut self, opcodes: &[u8]) -> Res<bool> {
        let value = fetch_u8(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("INT {value}");
        self.push(Value::Integer(value as i64))?;
        self.program_counter = checked_add(self.program_counter, 2)?;
        Ok(true)
    }

    fn int2(&mut self, opcodes: &[u8]) -> Res<bool> {
        let value = fetch_u16(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("INT {value}");
        self.push(Value::Integer(value as i64))?;
        self.program_counter = checked_add(self.program_counter, 3)?;
        Ok(true)
    }

    fn int8(&mut self, opcodes: &[u8]) -> Res<bool> {
        let value = fetch_u64(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("INT {value}");
        self.push(Value::Integer(value as i64))?;
        self.program_counter = checked_add(self.program_counter, 9)?;
        Ok(true)
    }

    fn jp2(&mut self, opcodes: &[u8]) -> Res<bool> {
        self.program_counter = fetch_u16(opcodes, checked_add(self.program_counter, 1)?)? as u32;
        dumpln!("JP {}", self.program_counter);
        Ok(true)
    }

    fn jp4(&mut self, opcodes: &[u8]) -> Res<bool> {
        self.program_counter = fetch_u32(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("JP {}", self.program_counter);
        Ok(true)
    }

    fn jf2(&mut self, opcodes: &[u8]) -> Res<bool> {
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
                Err(Error::UnexpectedType)
            }
        }
    }

    fn jf4(&mut self, opcodes: &[u8]) -> Res<bool> {
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
                Err(Error::UnexpectedType)
            }
        }
    }

    fn real(&mut self, opcodes: &[u8]) -> Res<bool> {
        let value = fetch_f64(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("REAL {value}");
        self.push(Value::Real(value))?;
        self.program_counter = checked_add(self.program_counter, 9)?;
        Ok(true)
    }

    fn boolean(&mut self, value: bool) -> Res<bool> {
        if value {
            dumpln!("TRUE");
        } else {
            dumpln!("FALSE");
        }
        self.push(Value::Boolean(value))?;
        self.program_counter = checked_add(self.program_counter, 1)?;
        Ok(true)
    }

    fn drop(&mut self) -> Res<bool> {
        dumpln!("DROP");
        self.pop()?;
        self.program_counter = checked_add(self.program_counter, 1)?;
        Ok(true)
    }

    fn void(&mut self) -> Res<bool> {
        dumpln!("VOID");
        self.push(Value::Void)?;
        self.program_counter = checked_add(self.program_counter, 1)?;
        Ok(true)
    }

    fn list(&mut self) -> Res<bool> {
        dumpln!("LIST");
        self.push(Value::Object(Rc::new(RefCell::new(Object::List(
            Vec::new(),
        )))))?;
        self.program_counter = checked_add(self.program_counter, 1)?;
        Ok(true)
    }

    fn ld1(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = fetch_u8(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("LD {index}");
        if self.locals + index as u32 >= self.stack.len() as u32 {
            return Err(Error::StackOverflow);
        }
        self.push(self.stack[(self.locals + index as u32) as usize].clone())?;
        self.program_counter = checked_add(self.program_counter, 2)?;
        Ok(true)
    }

    fn ld2(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = fetch_u16(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("LD {index}");
        if self.locals + index as u32 >= self.stack.len() as u32 {
            return Err(Error::StackOverflow);
        }
        self.push(self.stack[(self.locals + index as u32) as usize].clone())?;
        self.program_counter = checked_add(self.program_counter, 3)?;
        Ok(true)
    }

    fn ld4(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = fetch_u32(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("LD {index}");
        if self.locals + index >= self.stack.len() as u32 {
            return Err(Error::StackOverflow);
        }
        self.push(self.stack[(self.locals + index) as usize].clone())?;
        self.program_counter = checked_add(self.program_counter, 5)?;
        Ok(true)
    }

    fn st1(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = fetch_u8(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("ST {index}");
        if self.locals + index as u32 >= self.stack.len() as u32 {
            return Err(Error::StackOverflow);
        }
        self.stack[(self.locals + index as u32) as usize] = self.peek()?;
        self.program_counter = checked_add(self.program_counter, 2)?;
        Ok(true)
    }

    fn st2(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = fetch_u16(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("ST {index}");
        if self.locals + index as u32 >= self.stack.len() as u32 {
            return Err(Error::StackOverflow);
        }
        self.stack[(self.locals + index as u32) as usize] = self.peek()?;
        self.program_counter = checked_add(self.program_counter, 3)?;
        Ok(true)
    }

    fn st4(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = fetch_u32(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("ST {index}");
        if self.locals + index >= self.stack.len() as u32 {
            return Err(Error::StackOverflow);
        }
        self.stack[(self.locals + index) as usize] = self.peek()?;
        self.program_counter = checked_add(self.program_counter, 5)?;
        Ok(true)
    }

    fn error<T>(&mut self, m: String) -> Res<T> {
        self.message = Some(m);
        Err(Error::Custom)
    }

    fn index_get_list(&mut self, data: &Vec<Value>, key: Value) -> Res<Value> {
        match key {
            Value::Integer(index) => {
                if index >= 0 && (index as usize) < data.len() {
                    Ok(data[index as usize].clone())
                } else {
                    self.error(format!("Index out of range."))
                }
            }
            _ => self.error(format!("Can't to index list by {key}.")),
        }
    }

    fn index_get_object(&mut self, data: &Object, key: Value) -> Res<Value> {
        match data {
            Object::List(list) => self.index_get_list(list, key),
        }
    }

    fn index_get(&mut self, data: Value, key: Value) -> Res<Value> {
        match data {
            Value::Object(object) => {
                let object = object.borrow();
                self.index_get_object(&object, key)
            }
            _ => self.error(format!("Can't to index {data}.")),
        }
    }

    fn index_set_list(&mut self, data: &mut Vec<Value>, key: Value, value: Value) -> Res {
        match key {
            Value::Integer(index) => {
                if index >= 0 && (index as usize) < data.len() {
                    data[index as usize] = value;
                    Ok(())
                } else {
                    self.error(format!("Index out of range."))
                }
            }
            _ => self.error(format!("Can't to index list by {key}.")),
        }
    }

    fn index_set_object(&mut self, data: &mut Object, key: Value, value: Value) -> Res {
        match data {
            Object::List(list) => self.index_set_list(list, key, value),
        }
    }

    fn index_set(&mut self, data: Value, key: Value, value: Value) -> Res {
        match data {
            Value::Object(object) => {
                let mut object = object.borrow_mut();
                self.index_set_object(&mut object, key, value)
            }
            _ => self.error(format!("Can't to index {data}.")),
        }
    }

    fn get(&mut self) -> Res<bool> {
        dumpln!("GET");
        let key = self.pop()?;
        let data = self.pop()?;
        let result = self.index_get(data, key)?;
        self.push(result)?;
        self.program_counter = checked_add(self.program_counter, 1)?;
        Ok(true)
    }

    fn set(&mut self) -> Res<bool> {
        dumpln!("SET");
        let value = self.pop()?;
        let key = self.pop()?;
        let data = self.pop()?;
        self.index_set(data, key, value.clone())?;
        self.push(value)?;
        self.program_counter = checked_add(self.program_counter, 1)?;
        Ok(true)
    }

    fn call(&mut self, opcodes: &[u8]) -> Res<bool> {
        let params_count = fetch_u8(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("CALL {params_count}");
        if self.stack_pointer < params_count as u32 + 1 {
            return Err(Error::StackUnderflow);
        }
        let in_stack_offset = self.stack_pointer - params_count as u32 - 1;
        let address = self.stack[in_stack_offset as usize].clone();
        self.stack[in_stack_offset as usize] =
            Value::CallState(checked_add(self.program_counter, 2)?, self.locals);
        match address {
            Value::Pointer(address) => self.program_counter = address,
            _ => return self.error(format!("Expected address, found {address}")),
        }
        self.locals = self.stack_pointer - params_count as u32;
        let params_count_for_check = fetch_u8(opcodes, self.program_counter)?;
        if params_count != params_count_for_check {
            return self.error(format!(
                "Expected {params_count_for_check} function call arguments, found {params_count}."
            ));
        }
        let stack_size = fetch_u32(opcodes, checked_add(self.program_counter, 1)?)?;
        self.stack_pointer = checked_add(self.stack_pointer, stack_size)?;
        dumpln!("Call info: parameters count: {params_count}, stack_size: {stack_size}");
        self.program_counter = checked_add(self.program_counter, 5)?;
        Ok(true)
    }

    fn ptr(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = fetch_u32(opcodes, checked_add(self.program_counter, 1)?)?;
        dumpln!("PTR {index}");
        self.push(Value::Pointer(index))?;
        self.program_counter = checked_add(self.program_counter, 5)?;
        Ok(true)
    }

    pub fn step(&mut self, opcodes: &[u8]) -> Res<bool> {
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
            opcode::SET => self.set(),
            opcode::GET => self.get(),
            opcode::CALL => self.call(opcodes),
            opcode::PTR => self.ptr(opcodes),
            _ => Err(Error::UnknownOpcode),
        }
    }

    pub fn run(&mut self, opcodes: &[u8]) -> Res<Value> {
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
