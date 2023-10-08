#[derive(Debug)]
pub enum VMError {
    StackOverflow,
    StackUnderflow,
    OpcodeFetch,
    UnknownOpcode,
    BinaryOperation,
    DividingByZero,
}

pub type VMRes<T = ()> = Result<T, VMError>;
