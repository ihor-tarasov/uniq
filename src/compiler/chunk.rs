use crate::opcode;

use super::{opcodes::Opcodes, Pos, Res};
use std::collections::HashMap;

pub struct Chunk {
    opcodes: Opcodes,
    ranges: HashMap<u32, Pos>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            opcodes: Opcodes::new(),
            ranges: HashMap::new(),
        }
    }

    pub fn set_jf(&mut self, address: u32, value: u32) {
        if value <= 0xFFFF {
            self.opcodes[address] = opcode::JF2;
            (value as u16)
                .to_be_bytes()
                .iter()
                .cloned()
                .enumerate()
                .for_each(|(i, b)| self.opcodes[address + i as u32 + 1] = b);
        } else {
            self.opcodes[address] = opcode::JF4;
            value
                .to_be_bytes()
                .iter()
                .cloned()
                .enumerate()
                .for_each(|(i, b)| self.opcodes[address + i as u32 + 1] = b);
        }
    }

    pub fn set_jt(&mut self, address: u32, value: u32) {
        if value <= 0xFFFF {
            self.opcodes[address] = opcode::JT2;
            (value as u16)
                .to_be_bytes()
                .iter()
                .cloned()
                .enumerate()
                .for_each(|(i, b)| self.opcodes[address + i as u32 + 1] = b);
        } else {
            self.opcodes[address] = opcode::JT4;
            value
                .to_be_bytes()
                .iter()
                .cloned()
                .enumerate()
                .for_each(|(i, b)| self.opcodes[address + i as u32 + 1] = b);
        }
    }

    pub fn set_jp(&mut self, address: u32, value: u32) {
        if value <= 0xFFFF {
            self.opcodes[address] = opcode::JP2;
            (value as u16)
                .to_be_bytes()
                .iter()
                .cloned()
                .enumerate()
                .for_each(|(i, b)| self.opcodes[address + i as u32 + 1] = b);
        } else {
            self.opcodes[address] = opcode::JP4;
            value
                .to_be_bytes()
                .iter()
                .cloned()
                .enumerate()
                .for_each(|(i, b)| self.opcodes[address + i as u32 + 1] = b);
        }
    }

    pub fn integer(&mut self, value: &[u8]) -> Res {
        let value = std::str::from_utf8(value)?.parse::<u64>()?;
        if value <= 0xFF {
            self.opcodes.push(opcode::INT1)?;
            self.opcodes.push(value as u8)?;
        } else if value <= 0xFFFF {
            self.opcodes.push(opcode::INT2)?;
            self.opcodes.extend((value as u16).to_be_bytes())?;
        } else {
            self.opcodes.push(opcode::INT8)?;
            self.opcodes.extend(value.to_be_bytes())?;
        }
        Ok(())
    }

    pub fn real(&mut self, value: &[u8]) -> Res {
        let value = std::str::from_utf8(value)?.parse::<f64>()?;
        self.opcodes.push(opcode::REAL)?;
        self.opcodes.extend(value.to_be_bytes())
    }

    pub fn boolean(&mut self, value: bool) -> Res {
        self.opcodes
            .push(if value { opcode::TRUE } else { opcode::FALSE })
    }

    pub fn empty_address(&mut self) -> Res<u32> {
        let address = self.opcodes.len();
        self.opcodes.extend([0; 5])?;
        Ok(address)
    }

    pub fn len(&self) -> u32 {
        self.opcodes.len()
    }

    pub fn push(&mut self, opcode: u8) -> Res {
        self.opcodes.push(opcode)
    }

    pub fn call(&mut self, argc: u8) -> Res {
        self.opcodes.extend([opcode::CALL, argc])
    }

    pub fn store(&mut self, index: u32, is_local: bool) -> Res {
        if index <= u8::MAX as u32 {
            self.opcodes.extend([
                if is_local { opcode::ST1 } else { opcode::GS1 },
                index as u8,
            ])
        } else if index <= 0xFFFF {
            self.opcodes
                .push(if is_local { opcode::ST2 } else { opcode::GS2 })?;
            self.opcodes.extend((index as u16).to_be_bytes())
        } else {
            self.opcodes
                .push(if is_local { opcode::ST4 } else { opcode::GS4 })?;
            self.opcodes.extend(index.to_le_bytes())
        }
    }

    pub fn load(&mut self, index: u32, is_local: bool) -> Res {
        if index <= u8::MAX as u32 {
            self.opcodes.extend([
                if is_local { opcode::LD1 } else { opcode::GL1 },
                index as u8,
            ])
        } else if index <= 0xFFFF {
            self.opcodes
                .push(if is_local { opcode::LD2 } else { opcode::GL2 })?;
            self.opcodes.extend((index as u16).to_be_bytes())
        } else {
            self.opcodes
                .push(if is_local { opcode::LD4 } else { opcode::GL4 })?;
            self.opcodes.extend(index.to_le_bytes())
        }
    }

    pub fn ptr(&mut self, address: u32) -> Res {
        self.opcodes.push(opcode::PTR)?;
        self.opcodes.extend(address.to_be_bytes())
    }

    pub fn nat(&mut self, index: u32) -> Res {
        self.opcodes.push(opcode::NAT)?;
        self.opcodes.extend(index.to_be_bytes())
    }

    pub fn jump(&mut self, address: u32) -> Res {
        if address <= 0xFFFF {
            self.opcodes.push(opcode::JP2)?;
            self.opcodes.extend((address as u16).to_be_bytes())?;
            self.opcodes.extend([0; 2])
        } else {
            self.opcodes.push(opcode::JP4)?;
            self.opcodes.extend(address.to_be_bytes())
        }
    }

    pub fn start_function(&mut self, argc: u8) -> Res<u32> {
        self.opcodes.push(argc)?;
        let stack_size_address = self.opcodes.len();
        self.opcodes.extend([0; 4])?;
        Ok(stack_size_address)
    }

    pub fn start_global(&mut self) -> Res {
        self.opcodes.extend([0; 4])
    }

    pub fn write_u32_at(&mut self, address: u32, value: u32) {
        value
            .to_be_bytes()
            .iter()
            .cloned()
            .enumerate()
            .for_each(|(i, b)| self.opcodes[address + i as u32] = b);
    }

    pub fn push_pos(&mut self, pos: Pos) {
        self.ranges.insert(self.opcodes.len(), pos);
    }

    pub fn pop(&mut self) {
        self.opcodes.pop();
    }

    pub fn opcodes(&self) -> &[u8] {
        self.opcodes.as_slice()
    }

    pub fn pos(&self, address: u32) -> Option<&Pos> {
        self.ranges.get(&address)
    }
}
