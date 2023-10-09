use crate::{Value, SliceRead, utils, Chunk, State};

fn run(chunk: &Chunk, expected: Value) {
    let mut stack: [Value; 256] = std::array::from_fn(|_| Value::Void);
    let mut state = State::new(&mut stack);
    match state.run(&chunk.opcodes) {
        Ok(value) => {
            assert_eq!(value, expected);
        },
        Err(error) => {
            if let Some(message) = state.message() {
                panic!("Runtime error: {message}");
            } else {
                panic!("Runtime error: {error:?}");
            }
        },
    }
}

fn eval<'a, T>(code: T, expected: Value)
where
    T: Into<SliceRead<'a>>,
{
    if let Some(chunk) = utils::compile("stdin", code.into()) {
        run(&chunk, expected);
    }
}

#[test]
fn base_tests() {
    eval("2 + 2", Value::Integer(4));
    eval("2 + 2 * 2", Value::Integer(6));
    eval("2 * 0.5", Value::Real(1.0));
    eval("2 * 2 + 2 * 2", Value::Integer(8));
    eval("3 + 3 == 3 * 2 + 1", Value::Boolean(false));
    eval("2 < 3", Value::Boolean(true));
    eval("3 != 1", Value::Boolean(true));
    eval("3 + 3 >= 3 + 3", Value::Boolean(true));
    eval("3 + 3 > 3 * 2 - 1", Value::Boolean(true));
}
