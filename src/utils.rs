use crate::{
    compiler::{Chunk, Compiler},
    line,
    vm::{self, State, Value},
    SliceRead,
};
use std::{fmt::Write, ops::Range};

fn range_info<R>(path: &str, read: &mut R, range: Range<usize>) -> String
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

pub fn range_info_file(path: &str, range: Range<usize>) -> String {
    let mut read = std::io::BufReader::new(std::fs::File::open(path).unwrap());
    range_info(path, &mut read, range)
}

pub fn range_info_repl(source: &str, range: Range<usize>) -> String {
    let mut read = SliceRead::from(source);
    range_info("stdin", &mut read, range)
}

pub fn compile_file<'a>(
    compiler: &'a mut Compiler,
    source_id: usize,
    path: &str,
) -> Result<(), String> {
    let result = {
        let mut file = match std::fs::File::open(path) {
            Ok(file) => std::io::BufReader::new(file),
            Err(error) => {
                return Err(format!(
                    "Unablle to open file: \"{path}\", IOError: {error}"
                ))
            }
        };

        compiler.compile(source_id, &mut file)
    };

    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            let mut buffer = range_info_file(path, compiler.range());
            writeln!(buffer, "Compile error: {error}").unwrap();
            Err(buffer)
        }
    }
}

pub fn compile_repl<R>(
    compiler: &mut Compiler,
    source_id: usize,
    read: &mut R,
) -> Result<(), Option<String>>
where
    R: std::io::Read + std::io::Seek,
{
    match compiler.compile(source_id, read) {
        Ok(_) => Ok(()),
        Err(error) => {
            let range = compiler.range();
            if range.start == range.end {
                Err(None)
            } else {
                let mut buffer = range_info("stdin", read, range);
                writeln!(buffer, "Compile error: {error}").unwrap();
                Err(Some(buffer))
            }
        }
    }
}

pub fn new_stack<const STACK_SIZE: usize>() -> [Value; STACK_SIZE] {
    std::array::from_fn(|_| Value::Void)
}

fn collect_runtime_error_repl<T>(
    error: vm::Error,
    chunk: &Chunk,
    sources: &[String],
    state: &State,
) -> Result<T, String> {
    if let Some(pos) = chunk.pos(state.program_counter()) {
        let mut buffer = range_info_repl(sources[pos.source_id].as_str(), pos.range.clone());
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

fn collect_runtime_error_file<T>(
    error: vm::Error,
    chunk: &Chunk,
    paths: &[String],
    state: &State,
) -> Result<T, String> {
    if let Some(pos) = chunk.pos(state.program_counter()) {
        let mut buffer = range_info_file(paths[pos.source_id].as_str(), pos.range.clone());
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

pub fn run_repl(state: &mut State, sources: &[String], chunk: &Chunk) -> Result<Value, String> {
    match state.run(chunk.opcodes()) {
        Ok(value) => Ok(value),
        Err(error) => collect_runtime_error_repl(error, chunk, sources, state),
    }
}

pub fn run_file(state: &mut State, paths: &[String], chunk: &Chunk) -> Result<Value, String> {
    match state.run(chunk.opcodes()) {
        Ok(value) => Ok(value),
        Err(error) => collect_runtime_error_file(error, chunk, paths, state),
    }
}
