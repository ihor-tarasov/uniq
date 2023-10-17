#[derive(Debug)]
pub enum Error {
    StackOverflow,
    StackUnderflow,
    OpcodeFetch,
    UnknownOpcode,
    BinaryOperation,
    UnaryOperation,
    DividingByZero,
    UnexpectedType,
    AddressOverflow,
    FetchGlobal,
    Custom,
}

pub type Res<T = ()> = Result<T, Error>;
