use core::ops::Range;
use std::io::SeekFrom;

use crate::CompRes;

pub struct LineInfo {
    pub start: usize,
    pub number: usize,
}

pub fn create<R>(read: &mut R, start: usize) -> std::io::Result<LineInfo>
where
    R: std::io::Read,
{
    let mut line_number = 1;
    let mut line_start = 0;
    let mut offset = 0;
    let mut buf = [0u8; 1];
    loop {
        match read.read_exact(&mut buf) {
            Ok(_) => {
                if offset == start {
                    break;
                }

                offset += 1;

                if buf[0] == b'\n' {
                    line_number += 1;
                    line_start = offset;
                }
            }
            Err(error) => match error.kind() {
                std::io::ErrorKind::UnexpectedEof => break,
                _ => return Err(error),
            },
        }
    }
    Ok(LineInfo {
        start: line_start,
        number: line_number,
    })
}

pub fn print_line<R, W>(read: &mut R, start: usize, write: &mut W) -> CompRes
where
    R: std::io::Read + std::io::Seek,
    W: std::fmt::Write,
{
    read.seek(SeekFrom::Start(start as u64))?;

    let mut buf = [0u8; 1];
    loop {
        match read.read_exact(&mut buf) {
            Ok(_) => {
                if buf[0] != b'\n' && buf[0] != b'\r' {
                    write!(write, "{}", buf[0] as char)?;
                } else {
                    break;
                }
            }
            Err(error) => match error.kind() {
                std::io::ErrorKind::UnexpectedEof => break,
                _ => return Err(crate::CompilerError::IO(error)),
            },
        }
    }

    writeln!(write)?;
    Ok(())
}

pub fn mark_range<W>(line_start: usize, range: Range<usize>, write: &mut W) -> std::fmt::Result
where
    W: std::fmt::Write,
{
    for _ in line_start..range.start {
        write!(write, " ")?;
    }
    for _ in range {
        write!(write, "^")?;
    }
    writeln!(write)
}
