#[derive(Clone, Copy, PartialEq)]
pub enum Token {
    Integer,
    Real,
    Identifier,
    True, // 'true'
    False, // 'false'
    LeftParen, // '('
    RightParen, // ')'
    Plus, // '+'
    Minus, // '-'
    Asterisk, // '*'
    Slash, // '/'
    Equal, // '='
    Exclamation, // '!'
    ExclamationEqual, // '!='
    EqualEqual, // '=='
    Less, // '<'
    Greater, // '>'
    GreaterEqual, // '>='
    LessEqual, // '<='
    Unknown,
    End,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Integer => write!(f, "integer value"),
            Token::Real => write!(f, "real value"),
            Token::Identifier => write!(f, "identifier"),
            Token::True => write!(f, "'true'"),
            Token::False => write!(f, "'false'"),
            Token::LeftParen => write!(f, "'('"),
            Token::RightParen => write!(f, "')'"),
            Token::Plus => write!(f, "'+'"),
            Token::Minus => write!(f, "'-'"),
            Token::Asterisk => write!(f, "'*'"),
            Token::Slash => write!(f, "'/'"),
            Token::Equal => write!(f, "'='"),
            Token::Exclamation => write!(f, "'!'"),
            Token::ExclamationEqual => write!(f, "'!='"),
            Token::EqualEqual => write!(f, "'=='"),
            Token::Less => write!(f, "'<'"),
            Token::Greater => write!(f, "'>'"),
            Token::GreaterEqual => write!(f, "'>='"),
            Token::LessEqual => write!(f, "'<='"),
            Token::Unknown => write!(f, "unknown"),
            Token::End => write!(f, "end of code"),
        }
    }
}
