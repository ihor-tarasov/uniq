use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Instruction {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Addict,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equals,
    NotEquals,
    Less,
    Greater,
    LessEquals,
    GreaterEquals,
    End,
}
