use super::List;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Void,
    Boolean(bool),
    Integer(i64),
    Real(f64),
    Pointer(u32),
    Native(u32),
    CallState(u32, u32),
    List(List),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Void => write!(f, "()"),
            Value::Boolean(value) => write!(f, "{value}"),
            Value::Integer(value) => write!(f, "{value}"),
            Value::Real(value) => write!(f, "{value}"),
            Value::Pointer(value) => write!(f, "${value}"),
            Value::Native(value) => write!(f, "${value}"),
            Value::CallState(pc, locals) => write!(f, "(PC:{pc} LC:{locals})"),
            Value::List(list) => list.fmt(f),
        }
    }
}
