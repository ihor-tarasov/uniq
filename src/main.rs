use uniq::{Compiler, State, SliceIter, Value};

fn compile<T, I>(t: T) -> Option<Vec<u8>>
where
    T: Into<I>,
    I: Iterator<Item = std::io::Result<u8>>,
{
    let mut compiler = Compiler::new();
    match compiler.compile(t.into()) {
        Ok(_) => Some(compiler.finish()),
        Err(error) => {
            eprintln!("Compilation error: {error:?}");
            None
        }
    }
}

fn run(opcodes: &[u8]) {
    let mut stack: [Value; 256] = std::array::from_fn(|_| Value::Void);
    match State::new(&mut stack).run(&opcodes) {
        Ok(value) => println!("{value}"),
        Err(error) => eprintln!("Runtime error: {error:?}"),
    }
}

fn eval<'a, T>(code: T)
where
    T: Into<SliceIter<'a>>,
{
    if let Some(opcodes) = compile(code) {
        run(&opcodes)
    }
}

fn main() {
    eval("1 - 2.5");
}
