use std::fmt;

#[derive(PartialEq)]
pub enum Token {
    Integer(i32),
    Plus, // '+'
    Minus, // '-'
    Asterisk, // '*'
    Slash, // '/'
    Unknown(u8),
    ToBigInteger,
    End,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Integer(value) => write!(f, "{value}"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Asterisk => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Unknown(c) => write!(f, "{}", *c as char),
            Token::ToBigInteger => write!(f, "to big integer"),
            Token::End => write!(f, ""),
        }
    }
}
