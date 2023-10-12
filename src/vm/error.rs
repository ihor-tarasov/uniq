#[derive(Debug)]
pub enum Error {
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

pub type Res<T = ()> = Result<T, Error>;
