#[derive(Debug)]
pub enum CompilerError {
    Utf8(std::str::Utf8Error),
    ParseInt(std::num::ParseIntError),
    ParseFloat(std::num::ParseFloatError),
    IO(std::io::Error),
    Custom(Box<String>),
    Fmt(std::fmt::Error),
}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerError::Utf8(error) => write!(f, "{error}"),
            CompilerError::ParseInt(error) => write!(f, "{error}"),
            CompilerError::ParseFloat(error) => write!(f, "{error}"),
            CompilerError::IO(error) => write!(f, "{error}"),
            CompilerError::Custom(error) => write!(f, "{error}"),
            CompilerError::Fmt(error) => write!(f, "{error}"),
        }
    }
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

impl From<std::num::ParseFloatError> for CompilerError {
    fn from(value: std::num::ParseFloatError) -> Self {
        Self::ParseFloat(value)
    }
}

impl From<std::io::Error> for CompilerError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<std::fmt::Error> for CompilerError {
    fn from(value: std::fmt::Error) -> Self {
        Self::Fmt(value)
    }
}

#[macro_export]
macro_rules! raise {
    ($($arg:tt)*) => {{
        Err(crate::CompilerError::Custom(Box::new(format!($($arg)*))))
    }};
}
