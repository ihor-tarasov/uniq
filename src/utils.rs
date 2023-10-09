use crate::{line, Chunk, Compiler, SliceRead, State, Value};
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

pub fn compile<R>(path: &str, read: &mut R) -> Result<Chunk, String>
where
    R: std::io::Read + std::io::Seek,
{
    let mut compiler = Compiler::new(0);
    match compiler.compile(0, read) {
        Ok(_) => Ok(compiler.into_chunk()),
        Err(error) => {
            let mut buffer = range_info(path, read, compiler.range());
            writeln!(buffer, "Compile error: {error}").unwrap();
            Err(buffer)
        }
    }
}

pub fn run<R>(paths: &[&str], read: &mut R, chunk: &Chunk) -> Result<Value, String>
where
    R: std::io::Read + std::io::Seek,
{
    let mut stack: [Value; 256] = std::array::from_fn(|_| Value::Void);
    let mut state = State::new(&mut stack);
    match state.run(&chunk.opcodes) {
        Ok(value) => Ok(value),
        Err(error) => {
            if let Some(pos) = chunk.ranges.get(&state.program_counter()) {
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

pub fn eval<'a, T>(code: T)
where
    T: Into<SliceRead<'a>>,
{
    let mut read = code.into();
    match compile("stdin", &mut read) {
        Ok(chunk) => match run(&["stdin"], &mut read, &chunk) {
            Ok(value) => println!("{value}"),
            Err(error) => eprintln!("{error}"),
        },
        Err(error) => eprintln!("{error}"),
    }
}
