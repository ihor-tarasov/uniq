use crate::{token::TokenLocation, Instruction};

pub struct Binary {
    pub left: Node,
    pub right: Node,
    pub instruction: Instruction,
    pub location: TokenLocation,
}

pub enum Node {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Binary(Box<Binary>),
}

impl Node {
    pub fn new_boolean(value: bool) -> Self {
        Self::Boolean(value)
    }

    pub fn new_integer(value: i64) -> Self {
        Self::Integer(value)
    }

    pub fn new_float(value: f64) -> Self {
        Self::Float(value)
    }

    pub fn new_binary(
        left: Self,
        right: Self,
        instruction: Instruction,
        location: TokenLocation,
    ) -> Self {
        Self::Binary(Box::new(Binary {
            left,
            right,
            instruction,
            location,
        }))
    }
}
