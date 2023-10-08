#[derive(Clone, Copy)]
pub enum Token {
    Integer,
    Real,
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
