use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(PartialEq)]
pub enum Token {
    Integer(i64),
    Float(f64),
    Plus, // '+'
    Minus, // '-'
    Asterisk, // '*'
    Slash, // '/'
    Percent, // '%'
    Less, // '<'
    Greater, // '>'
    Equals, // '='
    EqualsEquals, // '=='
    LessEquals, // '<='
    GreaterEquals, // '>='
    Exclamation, // '!'
    ExclamationEquals, // '!='
    Unknown(u8),
    ToBigInteger,
    End,
}

#[derive(Default, Clone, Copy, Serialize, Deserialize)]
pub struct TokenLocation {
    pub line: u32,
    pub column: u32,
    pub length: u32,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Integer(value) => write!(f, "{value}"),
            Token::Float(value) => write!(f, "{value}"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Asterisk => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Less => write!(f, "<"),
            Token::Greater => write!(f, ">"),
            Token::Equals => write!(f, "="),
            Token::EqualsEquals => write!(f, "=="),
            Token::LessEquals => write!(f, "<="),
            Token::GreaterEquals => write!(f, ">="),
            Token::Exclamation => write!(f, "!"),
            Token::ExclamationEquals => write!(f, "!="),
            Token::Unknown(c) => write!(f, "{}", *c as char),
            Token::ToBigInteger => write!(f, "to big integer"),
            Token::End => write!(f, ""),
        }
    }
}
