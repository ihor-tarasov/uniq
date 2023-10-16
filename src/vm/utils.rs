use crate::opcode;

use super::{Res, Error};

pub fn checked_add(a: u32, b: u32) -> Res<u32> {
    a.checked_add(b).ok_or(Error::AddressOverflow)
}

pub fn checked_as(a: usize) -> Res<u32> {
    if a <= u32::MAX as usize {
        Ok(a as u32)
    } else {
        Err(Error::AddressOverflow)
    }
}

pub fn fetch_u8(opcodes: &[u8], offset: u32) -> Res<u8> {
    match opcodes.get(offset as usize) {
        Some(data) => Ok(*data),
        None => Err(Error::OpcodeFetch),
    }
}

pub fn fetch_u16(opcodes: &[u8], offset: u32) -> Res<u16> {
    Ok(u16::from_be_bytes([
        fetch_u8(opcodes, offset)?,
        fetch_u8(opcodes, checked_add(offset, 1)?)?,
    ]))
}

pub fn fetch_u32(opcodes: &[u8], offset: u32) -> Res<u32> {
    Ok(u32::from_be_bytes([
        fetch_u8(opcodes, offset)?,
        fetch_u8(opcodes, checked_add(offset, 1)?)?,
        fetch_u8(opcodes, checked_add(offset, 2)?)?,
        fetch_u8(opcodes, checked_add(offset, 3)?)?,
    ]))
}

pub fn fetch_u64(opcodes: &[u8], offset: u32) -> Res<u64> {
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

pub fn fetch_f64(opcodes: &[u8], offset: u32) -> Res<f64> {
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

pub fn dump_opcodes(opcodes: &[u8]) -> Res {
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
            opcode::INC => println!("INC"),
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
            opcode::JT2 => {
                let value = fetch_u16(opcodes, i)?;
                println!("JT {value}");
                i = checked_add(i, 4)?;
            }
            opcode::JT4 => {
                let value = fetch_u32(opcodes, i)?;
                println!("JT {value}");
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
            opcode::LD4 => {
                let value = fetch_u32(opcodes, i)?;
                println!("LD {value}");
                i = checked_add(i, 4)?;
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
            opcode::ST4 => {
                let value = fetch_u32(opcodes, i)?;
                println!("ST {value}");
                i = checked_add(i, 4)?;
            }
            opcode::GL1 => {
                let value = fetch_u8(opcodes, i)?;
                println!("GL {value}");
                i = checked_add(i, 1)?;
            }
            opcode::GL2 => {
                let value = fetch_u16(opcodes, i)?;
                println!("GL {value}");
                i = checked_add(i, 2)?;
            }
            opcode::GL4 => {
                let value = fetch_u32(opcodes, i)?;
                println!("GL {value}");
                i = checked_add(i, 4)?;
            }
            opcode::GS1 => {
                let value = fetch_u8(opcodes, i)?;
                println!("GS {value}");
                i = checked_add(i, 1)?;
            }
            opcode::GS2 => {
                let value = fetch_u16(opcodes, i)?;
                println!("GS {value}");
                i = checked_add(i, 2)?;
            }
            opcode::GS4 => {
                let value = fetch_u32(opcodes, i)?;
                println!("GS {value}");
                i = checked_add(i, 4)?;
            }
            opcode::PTR => {
                let value = fetch_u32(opcodes, i)?;
                println!("PTR {value}");
                i = checked_add(i, 4)?;
            }
            opcode::NAT => {
                let value = fetch_u32(opcodes, i)?;
                println!("NAT {value}");
                i = checked_add(i, 4)?;
            }
            _ => return Err(Error::UnknownOpcode),
        }
    }
    Ok(())
}
