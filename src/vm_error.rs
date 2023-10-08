#[derive(Debug)]
pub enum VMError {
    StackOverflow,
    StackUnderflow,
    OpcodeFetch,
    UnknownOpcode,
    BinaryOperation,
}

pub type VMRes<T = ()> = Result<T, VMError>;
