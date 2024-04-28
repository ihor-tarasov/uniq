use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Instruction {
    Integer(i64),
    Addict,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    End,
}
