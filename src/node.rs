use crate::Instruction;

pub enum Node {
    Integer(i64),
    Binary(Box<(Node, Node, Instruction)>),
}

impl Node {
    pub fn new_integer(value: i64) -> Self {
        Self::Integer(value)
    }

    pub fn new_binary(left: Self, right: Self, instruction: Instruction) -> Self {
        Self::Binary(Box::new((left, right, instruction)))
    }
}
