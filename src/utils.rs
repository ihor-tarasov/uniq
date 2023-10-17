use crate::{
    compiler::{Chunk, Compiler},
    line,
    natives::Natives,
    vm::{State, Value},
    SliceRead,
};
use std::{fmt::Write, ops::Range};

pub fn range_info<R>(path: &str, read: &mut R, range: Range<usize>) -> String
where
    R: std::io::Read + std::io::Seek,
{
    read.seek(std::io::SeekFrom::Start(0)).unwrap();
    let info = line::create(read, range.start).unwrap();
    let mut buffer = String::new();
    writeln!(buffer, "File \"{path}\", line: {}:", info.number).unwrap();
    line::print_line(read, info.start, &mut buffer).unwrap();
    line::mark_range(info.start, range, &mut buffer).unwrap();
    buffer
}

pub fn compile<'a, R>(compiler: &'a mut Compiler, path: &str, read: &mut R) -> Result<(), String>
where
    R: std::io::Read + std::io::Seek,
{
    match compiler.compile(0, read) {
        Ok(_) => Ok(()),
        Err(error) => {
            let mut buffer = range_info(path, read, compiler.range());
            writeln!(buffer, "Compile error: {error}").unwrap();
            Err(buffer)
        }
    }
}

pub fn compile_eof<R>(
    compiler: &mut Compiler,
    path: &str,
    read: &mut R,
) -> Result<(), Option<String>>
where
    R: std::io::Read + std::io::Seek,
{
    match compiler.compile(0, read) {
        Ok(_) => Ok(()),
        Err(error) => {
            let range = compiler.range();
            if range.start == range.end {
                Err(None)
            } else {
                let mut buffer = range_info(path, read, range);
                writeln!(buffer, "Compile error: {error}").unwrap();
                Err(Some(buffer))
            }
        }
    }
}

pub fn run<R>(
    start: u32,
    paths: &[&str],
    read: &mut R,
    chunk: &Chunk,
    natives: &Natives,
) -> Result<Value, String>
where
    R: std::io::Read + std::io::Seek,
{
    let mut stack: [Value; 256] = std::array::from_fn(|_| Value::Void);
    let mut state = State::new(start, &mut stack, natives);
    match state.run(chunk.opcodes()) {
        Ok(value) => Ok(value),
        Err(error) => {
            if let Some(pos) = chunk.pos(state.program_counter()) {
                let mut buffer = range_info(paths[pos.source_id], read, pos.range.clone());
                if let Some(message) = state.message() {
                    writeln!(buffer, "Runtime error: {message}").unwrap();
                } else {
                    writeln!(buffer, "Runtime error: {error:?}").unwrap();
                }
                Err(buffer)
            } else {
                let mut buffer = String::new();
                if let Some(message) = state.message() {
                    writeln!(buffer, "Runtime error: {message}").unwrap();
                } else {
                    writeln!(buffer, "Runtime error: {error:?}").unwrap();
                }
                Err(buffer)
            }
        }
    }
}

pub fn eval<'a, T>(code: T, natives: &Natives)
where
    T: Into<SliceRead<'a>>,
{
    let mut read = code.into();
    let mut compiler = Compiler::new(0, natives);
    match compile(&mut compiler, "stdin", &mut read) {
        Ok(_) => {
            compiler.finish();
            match run(0, &["stdin"], &mut read, compiler.chunk(), natives) {
                Ok(value) => println!("{value}"),
                Err(error) => eprintln!("{error}"),
            }
        }
        Err(error) => eprintln!("{error}"),
    }
}

pub fn eval_eof<'a, T>(code: T, natives: &Natives) -> bool
where
    T: Into<SliceRead<'a>>,
{
    let mut read = code.into();
    let mut compiler = Compiler::new(0, natives);
    match compile_eof(&mut compiler, "stdin", &mut read) {
        Ok(_) => {
            compiler.finish();
            match run(0, &["stdin"], &mut read, compiler.chunk(), natives) {
                Ok(value) => {
                    println!("{value}");
                    false
                }
                Err(error) => {
                    eprintln!("{error}");
                    false
                }
            }
        }
        Err(error) => {
            if let Some(error) = error {
                eprintln!("{error}");
                false
            } else {
                true
            }
        }
    }
}
