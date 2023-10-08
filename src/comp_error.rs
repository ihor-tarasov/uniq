#[derive(Debug)]
pub enum CompilerError {
    Utf8(std::str::Utf8Error),
    ParseInt(std::num::ParseIntError),
    IO(std::io::Error),
    Custom(Box<String>),
}

pub type CompRes<T = ()> = Result<T, CompilerError>;

impl From<std::str::Utf8Error> for CompilerError {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::Utf8(value)
    }
}

impl From<std::num::ParseIntError> for CompilerError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::ParseInt(value)
    }
}

impl From<std::io::Error> for CompilerError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

#[macro_export]
macro_rules! raise {
    ($($arg:tt)*) => {{
        Err(crate::CompilerError::Custom(Box::new(format!($($arg)*))))
    }};
}
