#[derive(Clone, Copy)]
pub enum Token {
    Integer,
    Plus,
    Minus,
    Asterisk,
    Unknown,
    End,
}
