pub enum Operator {
    Addict,
    Subtract,
    Multiply,
    Divide,
}

pub enum Node {
    Integer(i32),
    Binary(Box<(Node, Node, Operator)>),
}

impl Node {
    pub fn new_integer(value: i32) -> Self {
        Self::Integer(value)
    }

    pub fn new_binary(left: Self, right: Self, operator: Operator) -> Self {
        Self::Binary(Box::new((left, right, operator)))
    }
}
