use std::{cell::RefCell, rc::Rc};

use crate::{utils, SliceRead, vm::Value, library};

fn eval<'a, T>(code: T, expected: Value)
where
    T: Into<SliceRead<'a>>,
{
    let natives = library::load();
    let mut read = code.into();
    match utils::compile("stdin", &mut read, &natives) {
        Ok(chunk) => match utils::run(&["stdin"], &mut read, &chunk, &natives) {
            Ok(value) => assert_eq!(value, expected),
            Err(error) => panic!("{error}"),
        },
        Err(error) => panic!("{error}"),
    }
}

#[test]
fn arithmetic_tests() {
    eval("2 + 2", Value::Integer(4));
    eval("2 + 2 * 2", Value::Integer(6));
    eval("2 * 0.5", Value::Real(1.0));
    eval("2 * 2 + 2 * 2", Value::Integer(8));
}

#[test]
fn comparison_tests() {
    eval("3 + 3 == 3 * 2 + 1", Value::Boolean(false));
    eval("2 < 3", Value::Boolean(true));
    eval("3 != 1", Value::Boolean(true));
    eval("3 + 3 >= 3 + 3", Value::Boolean(true));
    eval("3 + 3 > 3 * 2 - 1", Value::Boolean(true));
}

#[test]
fn if_tests() {
    eval("if 2 == 4 { 2 }", Value::Void);
    eval("if 2 == 2 { 2 }", Value::Integer(2));
    eval("if 2 == 2 { 2 } else { 3 }", Value::Integer(2));
    eval("if 2 == 4 { 2 } else { 3 }", Value::Integer(3));
    eval("if 2 == 4 { 2 } else if 2 == 1 { 3 }", Value::Void);
    eval("if 2 == 2 { 2 } else if 2 == 3 { 3 }", Value::Integer(2));
    eval("if 2 == 2 { 2 } else if 2 == 3 { 3 }", Value::Integer(2));
    eval(
        "if 2 == 4 { 2 } else if 2 == 3 { 3 } else { 4 }",
        Value::Integer(4),
    );
}

#[test]
fn list_tests() {
    eval(
        "[]",
        Value::List(Rc::new(RefCell::new(Vec::new()))),
    );
    eval(
        "[2]",
        Value::List(Rc::new(RefCell::new(vec![Value::Integer(2)]))),
    );
    eval(
        "[2 3]",
        Value::List(Rc::new(RefCell::new(vec![Value::Integer(2), Value::Integer(3)]))),
    );
    eval(
        "[2, 3]",
        Value::List(Rc::new(RefCell::new(vec![Value::Integer(2), Value::Integer(3)]))),
    );
    eval(
        "[2, 3,]",
        Value::List(Rc::new(RefCell::new(vec![Value::Integer(2), Value::Integer(3)]))),
    );
    eval(
        "[2, 3] + 4",
        Value::List(Rc::new(RefCell::new(vec![Value::Integer(2), Value::Integer(3), Value::Integer(4)]))),
    );
}

#[test]
fn indexing_tests() {
    eval("let a = [1, 2, 3]; a[1]", Value::Integer(2));
    eval("let a = [1, 2, 3]; a[0] = 10; a[0]", Value::Integer(10));
    eval("let get_array = |n| { let a = []; let i = 0; while i < n { a = a + (i + 1); i = i + 1; }; a }; get_array(5)[1];", Value::Integer(2));
}

#[test]
fn function_tests() {
    eval("let a = |b| { b + 1 }; a(4)", Value::Integer(5));
    eval("let power = |n| { n * n }; power(power(10))", Value::Integer(10_000));
    eval("let factorial = |n| {
                      let fact = 1;
                      let i = 2;
                      while i <= n {
                              fact = fact * i;
                              i = i + 1;
                      }
                      fact
              };
              factorial(6)", Value::Integer(720));
}
