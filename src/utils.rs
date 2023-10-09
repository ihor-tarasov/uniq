use crate::{Chunk, Compiler, line, Value, State, SliceRead};
use std::fmt::Write;

pub fn compile<R>(path: &str, mut read: R) -> Option<Chunk>
where
    R: std::io::Read + std::io::Seek,
{
    let mut compiler = Compiler::new(0);
    match compiler.compile(0, &mut read) {
        Ok(_) => Some(compiler.into_chunk()),
        Err(error) => {
            let range = compiler.range();
            read.seek(std::io::SeekFrom::Start(0)).unwrap();
            let info = line::create(&mut read, range.start).unwrap();
            let mut buffer = String::new();
            writeln!(buffer, "File \"{path}\", line: {}:", info.number).unwrap();
            line::print_line(&mut read, info.start, &mut buffer).unwrap();
            line::mark_range(info.start, range, &mut buffer).unwrap();
            writeln!(buffer, "Compile error: {error:?}").unwrap();
            eprintln!("{buffer}");
            None
        }
    }
}

pub fn run(chunk: &Chunk) {
    let mut stack: [Value; 256] = std::array::from_fn(|_| Value::Void);
    let mut state = State::new(&mut stack);
    match state.run(&chunk.opcodes) {
        Ok(value) => println!("{value}"),
        Err(error) => {
            if let Some(message) = state.message() {
                eprintln!("Runtime error: {message}")
            } else {
                eprintln!("Runtime error: {error:?}")
            }
        },
    }
}

pub fn eval<'a, T>(code: T)
where
    T: Into<SliceRead<'a>>,
{
    if let Some(chunk) = compile("stdin", code.into()) {
        run(&chunk)
    }
}
