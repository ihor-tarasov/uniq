#[derive(Clone, Copy, PartialEq)]
pub enum Token {
    Integer,
    Real,
    Identifier,
    True, // 'true'
    False, // 'false'
    If, // 'if'
    Else, // 'else'
    Let, // 'let'
    While, // 'while'
    For, // 'for'
    VerticalBar, // '|'
    Comma, // ','
    LeftParen, // '('
    RightParen, // ')'
    LeftBrace, // '{'
    RightBrace, // '}'
    LeftBracket, // '['
    RightBracket, // ']'
    Semicolon, // ';'
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
            Token::If => write!(f, "'if'"),
            Token::Else => write!(f, "'else'"),
            Token::Let => write!(f, "'let'"),
            Token::While => write!(f, "'while'"),
            Token::For => write!(f, "'for'"),
            Token::LeftBracket => write!(f, "'['"),
            Token::RightBracket => write!(f, "']'"),
            Token::VerticalBar => write!(f, "'|'"),
            Token::Comma => write!(f, "','"),
            Token::LeftParen => write!(f, "'('"),
            Token::RightParen => write!(f, "')'"),
            Token::LeftBrace => write!(f, "'{{'"),
            Token::RightBrace => write!(f, "'}}'"),
            Token::Semicolon => write!(f, "';'"),
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
