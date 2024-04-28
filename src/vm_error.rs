use std::fmt;

#[derive(Debug)]
pub enum VMError {
    StackOverflow,
    StackUnderflow,
    ProgramCounterOutOfBounds,
    DividingByZero,
}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VMError::StackOverflow => write!(f, "Stack overflow"),
            VMError::StackUnderflow => write!(f, "Stack underflow"),
            VMError::ProgramCounterOutOfBounds => write!(f, "Program counter out of bounds"),
            VMError::DividingByZero => write!(f, "Dividing by zero"),
        }
    }
}
