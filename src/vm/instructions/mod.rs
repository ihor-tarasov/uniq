use std::{rc::Rc, cell::RefCell};

use crate::{vm::Error, opcode};

use super::{State, Res, Value, utils};

mod stack;
mod arithmetic;
mod comparison;
mod call;
mod constant;
mod jump;
mod load_store;
mod get_set;

const DUMP_OPCODE: bool = false;
const DUMP_STACK: bool = false;
const DUMP_OPCODES: bool = false;

#[macro_export]
macro_rules! dumpln {
    () => {
        if crate::vm::instructions::DUMP_OPCODE {
            println!();
        }
    };
    ($($arg:tt)*) => {{
        if crate::vm::instructions::DUMP_OPCODE {
            println!($($arg)*);
        }
    }};
}

impl<'a> State<'a> {
    fn bin<F>(&mut self, f: F) -> Res<bool>
    where
        F: Fn(&mut Self, Value, Value) -> Res<Value>,
    {
        let r = self.pop()?;
        let l = self.pop()?;
        let res = f(self, l, r)?;
        self.push(res)?;
        self.program_counter = utils::checked_add(self.program_counter, 1)?;
        Ok(true)
    }

    fn unary<F>(&mut self, f: F) -> Res<bool>
    where
        F: Fn(&mut Self, Value) -> Res<Value>,
    {
        let value = self.pop()?;
        let res = f(self, value)?;
        self.push(res)?;
        self.program_counter = utils::checked_add(self.program_counter, 1)?;
        Ok(true)
    }

    fn list(&mut self) -> Res<bool> {
        dumpln!("LIST");
        self.push(Value::List(Rc::new(RefCell::new(Vec::new(),
        ))))?;
        self.program_counter = utils::checked_add(self.program_counter, 1)?;
        Ok(true)
    }

    fn ptr(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u32(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("PTR {index}");
        self.push(Value::Pointer(index))?;
        self.program_counter = utils::checked_add(self.program_counter, 5)?;
        Ok(true)
    }

    fn nat(&mut self, opcodes: &[u8]) -> Res<bool> {
        let index = utils::fetch_u32(opcodes, utils::checked_add(self.program_counter, 1)?)?;
        dumpln!("NAT {index}");
        self.push(Value::Native(index))?;
        self.program_counter = utils::checked_add(self.program_counter, 5)?;
        Ok(true)
    }

    pub fn step(&mut self, opcodes: &[u8]) -> Res<bool> {
        let opcode = utils::fetch_u8(opcodes, self.program_counter)?;
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
            opcode::NAT => self.nat(opcodes),
            opcode::INC => self.unary(Self::inc),
            opcode::JT2 => self.jt2(opcodes),
            opcode::JT4 => self.jt4(opcodes),
            _ => Err(Error::UnknownOpcode),
        }
    }

    pub fn run(&mut self, opcodes: &[u8]) -> Res<Value> {
        if DUMP_OPCODES {
            println!("# OPCODES DUMP");
            utils::dump_opcodes(opcodes)?;
        }
        if DUMP_OPCODE {
            println!("# RUNTIME DUMP");
        }

        let stack_size = utils::fetch_u32(opcodes, 0)?;
        self.stack_pointer = stack_size;
        self.program_counter = utils::checked_add(self.program_counter, 4)?;

        while self.step(opcodes)? {
            if DUMP_STACK {
                self.dump_stack();
            }
        }
        self.pop()
    }
}
