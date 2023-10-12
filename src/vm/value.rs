use std::{rc::Rc, cell::RefCell, fmt};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Void,
    Boolean(bool),
    Integer(i64),
    Real(f64),
    Pointer(u32),
    CallState(u32, u32),
    Object(Rc<RefCell<Object>>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Void => write!(f, "()"),
            Value::Boolean(value) => write!(f, "{value}"),
            Value::Integer(value) => write!(f, "{value}"),
            Value::Real(value) => write!(f, "{value}"),
            Value::Pointer(value) => write!(f, "{value}"),
            Value::CallState(pc, locals) => write!(f, "(PC:{pc} LC:{locals})"),
            Value::Object(object) => write!(f, "{}", object.borrow()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Object {
    List(Vec<Value>),
}

impl Object {
    pub fn push(&mut self, value: Value) {
        match self {
            Object::List(list) => list.push(value),
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::List(list) => {
                write!(f, "[")?;
                let mut iter = list.iter();
                if let Some(value) = iter.next() {
                    write!(f, "{value}")?;
                    for value in iter {
                        write!(f, ", {value}")?;
                    }
                }
                write!(f, "]")
            }
        }
    }
}
