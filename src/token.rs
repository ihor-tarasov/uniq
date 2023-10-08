#[derive(Clone, Copy)]
pub enum Token {
    Integer,
    Real,
    Plus, // '+'
    Minus, // '-'
    Asterisk, // '*'
    Equal, // '='
    EqualEqual, // "=="
    Unknown,
    End,
}
