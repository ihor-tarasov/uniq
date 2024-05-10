use crate::token::{Token, TokenLocation};

pub struct Lexer<I> {
    iter: I,
    current: Option<u8>,
    offset: u32,
    location: TokenLocation,
}

impl<I> Lexer<I>
where
    I: Iterator<Item = u8>,
{
    pub fn new(mut iter: I) -> Self {
        Self {
            current: iter.next(),
            iter,
            offset: 0,
            location: TokenLocation::default(),
        }
    }

    fn advance(&mut self) {
        self.current = self.iter.next();
        self.offset += 1;
    }

    fn single(&mut self, kind: Token) -> Token {
        self.advance();
        kind
    }

    fn less(&mut self) -> Token {
        self.advance();
        match self.current {
            Some(b'=') => self.single(Token::LessEquals),
            _ => Token::Less,
        }
    }

    fn greater(&mut self) -> Token {
        self.advance();
        match self.current {
            Some(b'=') => self.single(Token::GreaterEquals),
            _ => Token::Greater,
        }
    }

    fn equals(&mut self) -> Token {
        self.advance();
        match self.current {
            Some(b'=') => self.single(Token::EqualsEquals),
            _ => Token::Equals,
        }
    }

    fn exclamation(&mut self) -> Token {
        self.advance();
        match self.current {
            Some(b'=') => self.single(Token::ExclamationEquals),
            _ => Token::Exclamation,
        }
    }

    fn whitespaces(&mut self) {
        while let Some(c) = self.current {
            if c.is_ascii_whitespace() {
                self.current = self.iter.next();
                if c == b'\n' {
                    self.location.line += 1;
                    self.offset = 0;
                } else {
                    self.offset += 1;
                }
            } else {
                break;
            }
        }
    }

    fn real(&mut self, first_part: i64) -> Token {
        let mut accumulator = first_part as f64;
        let mut denominator = 0.0;
        while let Some(c) = self.current {
            if c.is_ascii_digit() {
                denominator += 10.0;
                accumulator += ((c - b'0') as f64) / denominator;
                self.advance();
            } else {
                break;
            }
        }
        Token::Float(accumulator)
    }

    fn number(&mut self) -> Token {
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
            } else if c == b'.' {
                self.advance();
                return self.real(accumulator);
            } else {
                break;
            }
        }
        Token::Integer(accumulator)
    }

    pub fn next(&mut self) -> Token {
        self.whitespaces();
        self.location.column = self.offset;
        let token = if let Some(c) = self.current {
            match c {
                b'+' => self.single(Token::Plus),
                b'-' => self.single(Token::Minus),
                b'*' => self.single(Token::Asterisk),
                b'/' => self.single(Token::Slash),
                b'%' => self.single(Token::Percent),
                b'!' => self.exclamation(),
                b'<' => self.less(),
                b'>' => self.greater(),
                b'=' => self.equals(),
                b'0'..=b'9' => self.number(),
                _ => self.single(Token::Unknown(c)),
            }
        } else {
            Token::End
        };
        self.location.length = self.offset - self.location.column;
        token
    }

    pub fn location(&self) -> TokenLocation {
        self.location
    }
}
