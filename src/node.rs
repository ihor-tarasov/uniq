pub enum Operator {
    Addict,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

pub enum Node {
    Integer(i64),
    Binary(Box<(Node, Node, Operator)>),
}

impl Node {
    pub fn new_integer(value: i64) -> Self {
        Self::Integer(value)
    }

    pub fn new_binary(left: Self, right: Self, operator: Operator) -> Self {
        Self::Binary(Box::new((left, right, operator)))
    }
}
