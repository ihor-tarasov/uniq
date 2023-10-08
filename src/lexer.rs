use crate::Token;

pub struct Lexer<I> {
    iter: I,
    current: Option<u8>,
    offset: usize,
}

impl<I> Lexer<I>
where
    I: Iterator<Item = std::io::Result<u8>>,
{
    pub fn new(mut iter: I) -> std::io::Result<Self> {
        Ok(Self {
            current: match iter.next() {
                Some(r) => Some(r?),
                None => None,
            },
            offset: 0,
            iter,
        })
    }

    fn advance(&mut self) -> std::io::Result<()> {
        match self.iter.next() {
            Some(data) => {
                self.current = Some(data?);
                self.offset += 1;
                Ok(())
            }
            None => {
                self.current = None;
                Ok(())
            }
        }
    }

    fn integer(&mut self, buf: &mut Vec<u8>) -> std::io::Result<Token> {
        buf.clear();
        let mut has_dot = false;
        while let Some(c) = self.current {
            if c.is_ascii_digit() {
                buf.push(c);
                self.advance()?;
            } else if c == b'.' {
                if has_dot {
                    break;
                } else {
                    has_dot = true;
                    buf.push(c);
                    self.advance()?;
                }
            } else {
                break;
            }
        }
        Ok(if has_dot { Token::Real } else { Token::Integer })
    }

    fn single(&mut self, token: Token) -> std::io::Result<Token> {
        self.advance()?;
        Ok(token)
    }

    fn double_equal(&mut self) -> std::io::Result<Token> {
        self.advance()?;
        if let Some(c) = self.current {
            match c {
                b'=' => self.single(Token::EqualEqual),
                _ => Ok(Token::Equal)
            }
        } else {
            Ok(Token::Equal)
        }
    }

    pub fn lex(&mut self, buf: &mut Vec<u8>) -> std::io::Result<Token> {
        if let Some(c) = self.current {
            match c {
                b'0'..=b'9' => self.integer(buf),
                b'+' => self.single(Token::Plus),
                b'-' => self.single(Token::Minus),
                b'*' => self.single(Token::Asterisk),
                b'=' => self.double_equal(),
                _ => self.single(Token::Unknown),
            }
        } else {
            Ok(Token::End)
        }
    }

    pub fn skip_whitespaces(&mut self) -> std::io::Result<()> {
        while let Some(c) = self.current {
            if c.is_ascii_whitespace() {
                self.advance()?;
            } else {
                break;
            }
        }
        Ok(())
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}
