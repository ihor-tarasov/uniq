use crate::{Value, SliceRead, utils};

fn eval<'a, T>(code: T, expected: Value)
where
    T: Into<SliceRead<'a>>,
{
    let mut read = code.into();
    match utils::compile("stdin", &mut read) {
        Ok(chunk) => match utils::run(&["stdin"], &mut read, &chunk) {
            Ok(value) => assert_eq!(value, expected),
            Err(error) => panic!("{error}"),
        },
        Err(error) => panic!("{error}"),
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
