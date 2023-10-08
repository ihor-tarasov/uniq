#[derive(Clone, Copy)]
pub enum Token {
    Integer,
    Real,
    Plus,
    Minus,
    Asterisk,
    Unknown,
    End,
}
