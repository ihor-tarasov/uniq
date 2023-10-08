use crate::{Value, State, SliceIter, Compiler};

fn compile<T, I>(t: T) -> Option<Vec<u8>>
where
    T: Into<I>,
    I: Iterator<Item = std::io::Result<u8>>,
{
    let mut compiler = Compiler::new();
    match compiler.compile(t.into()) {
        Ok(_) => Some(compiler.finish()),
        Err(error) => {
            assert!(false, "Compilation error: {error:?}");
            None
        }
    }
}

fn run(opcodes: &[u8]) -> Value {
    let mut stack: [Value; 256] = std::array::from_fn(|_| Value::Void);
    match State::new(&mut stack).run(&opcodes) {
        Ok(value) => value,
        Err(error) => {
            panic!("Runtime error: {error:?}")
        },
    }
}

fn eval<'a, T>(code: T, expected: Value)
where
    T: Into<SliceIter<'a>>,
{
    if let Some(opcodes) = compile(code) {
        assert_eq!(run(&opcodes), expected)
    }
}

#[test]
fn base_tests() {
    eval("2 + 2", Value::Integer(4));
    eval("2 + 2 * 2", Value::Integer(6));
    eval("2 * 0.5", Value::Real(1.0));
    eval("2 * 2 + 2 * 2", Value::Integer(8));
    eval("3 + 3 == 3 * 2 + 1", Value::Boolean(false));
}
