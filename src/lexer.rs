use crate::token::Token;

pub struct Lexer<I> {
    iter: I,
    current: Option<u8>,
}

impl<I> Lexer<I>
where
    I: Iterator<Item = u8>,
{
    pub fn new(mut iter: I) -> Self {
        Self {
            current: iter.next(),
            iter,
        }
    }

    fn advance(&mut self) {
        self.current = self.iter.next();
    }

    fn single(&mut self, kind: Token) -> Token {
        self.advance();
        kind
    }

    fn whitespaces(&mut self) {
        while let Some(c) = self.current {
            if c.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn integer(&mut self) -> Token {
        let mut accumulator = 0i64;
        while let Some(c) = self.current {
            if c.is_ascii_digit() {
                let (next, overflow) = accumulator.overflowing_mul(10);
                if overflow {
                    return Token::ToBigInteger;
                }
                let (next, overflow) = next.overflowing_add((c - b'0') as i64);
                if overflow {
                    return Token::ToBigInteger;
                }
                accumulator = next;
                self.advance();
            } else {
                break;
            }
        }
        Token::Integer(accumulator)
    }

    pub fn next(&mut self) -> Token {
        self.whitespaces();
        if let Some(c) = self.current {
            match c {
                b'+' => self.single(Token::Plus),
                b'-' => self.single(Token::Minus),
                b'*' => self.single(Token::Asterisk),
                b'/' => self.single(Token::Slash),
                b'0'..=b'9' => self.integer(),
                _ => self.single(Token::Unknown(c)),
            }
        } else {
            Token::End
        }
    }
}
