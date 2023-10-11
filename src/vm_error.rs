#[derive(Debug)]
pub enum VMError {
    StackOverflow,
    StackUnderflow,
    OpcodeFetch,
    UnknownOpcode,
    BinaryOperation,
    DividingByZero,
    UnexpectedType,
    AddressOverflow,
    Custom,
}

pub type VMRes<T = ()> = Result<T, VMError>;
