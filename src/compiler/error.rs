#[derive(Debug)]
pub enum Error {
    Utf8(std::str::Utf8Error),
    ParseInt(std::num::ParseIntError),
    ParseFloat(std::num::ParseFloatError),
    IO(std::io::Error),
    Custom(Box<String>),
    Fmt(std::fmt::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Utf8(error) => write!(f, "{error}"),
            Self::ParseInt(error) => write!(f, "{error}"),
            Self::ParseFloat(error) => write!(f, "{error}"),
            Self::IO(error) => write!(f, "{error}"),
            Self::Custom(error) => write!(f, "{error}"),
            Self::Fmt(error) => write!(f, "{error}"),
        }
    }
}

pub type Res<T = ()> = Result<T, Error>;

impl From<std::str::Utf8Error> for Error {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::Utf8(value)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::ParseInt(value)
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(value: std::num::ParseFloatError) -> Self {
        Self::ParseFloat(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<std::fmt::Error> for Error {
    fn from(value: std::fmt::Error) -> Self {
        Self::Fmt(value)
    }
}

#[macro_export]
macro_rules! raise {
    ($($arg:tt)*) => {{
        Err(crate::compiler::Error::Custom(Box::new(format!($($arg)*))))
    }};
}
