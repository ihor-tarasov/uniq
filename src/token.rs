#[derive(Clone, Copy)]
pub enum Token {
    Integer,
    Real,
    Identifier,
    True, // 'true'
    False, // 'false'
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
