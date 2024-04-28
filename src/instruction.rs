use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Instruction {
    Integer(i32),
    Addict,
    Subtract,
    Multiply,
    Divide,
    End,
}
