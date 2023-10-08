use uniq::{Compiler, State, SliceIter, Value, Chunk};

fn compile<T, I>(t: T) -> Option<Chunk>
where
    T: Into<I>,
    I: Iterator<Item = std::io::Result<u8>>,
{
    let mut compiler = Compiler::new(0);
    match compiler.compile(0, t.into()) {
        Ok(_) => Some(compiler.into_chunk()),
        Err(error) => {
            eprintln!("Compilation error: {error:?}");
            None
        }
    }
}

fn run(chunk: &Chunk) {
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

fn eval<'a, T>(code: T)
where
    T: Into<SliceIter<'a>>,
{
    if let Some(chunk) = compile(code) {
        run(&chunk)
    }
}

fn main() {
    eval("3 + 3 == 3 * 2 == 3");
}
