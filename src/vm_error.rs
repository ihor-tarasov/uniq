#[derive(Debug)]
pub enum VMError {
    StackOverflow,
    StackUnderflow,
    OpcodeFetch,
    UnknownOpcode,
}

pub type VMRes<T = ()> = Result<T, VMError>;
