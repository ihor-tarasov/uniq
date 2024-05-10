use std::fmt;

use serde::{Deserialize, Serialize};

use crate::identifiers::{IdentifierId, Identifiers};

#[derive(PartialEq)]
pub enum Token {
    Integer(i64),
    Float(f64),
    Plus,              // '+'
    Minus,             // '-'
    Asterisk,          // '*'
    Slash,             // '/'
    Percent,           // '%'
    Less,              // '<'
    Greater,           // '>'
    Equals,            // '='
    EqualsEquals,      // '=='
    LessEquals,        // '<='
    GreaterEquals,     // '>='
    Exclamation,       // '!'
    ExclamationEquals, // '!='
    True,              // 'true'
    False,             // 'false'
    Identifier(IdentifierId),
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

fn write_u8_slice(f: &mut fmt::Formatter, slice: &[u8]) -> fmt::Result {
    for c in slice {
        write!(f, "{}", *c as char)?;
    }
    Ok(())
}

pub struct TokenWriter<'a, 'b> {
    token: &'b Token,
    identifiers: &'a Identifiers,
}

impl<'a, 'b> TokenWriter<'a, 'b> {
    pub fn new(token: &'b Token, identifiers: &'a Identifiers) -> Self {
        Self { token, identifiers }
    }
}

impl<'a, 'b> fmt::Display for TokenWriter<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.token {
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
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Identifier(id) => write_u8_slice(f, self.identifiers.get(id)),
            Token::Unknown(c) => write!(f, "{}", *c as char),
            Token::ToBigInteger => write!(f, "to big integer"),
            Token::End => write!(f, ""),
        }
    }
}
