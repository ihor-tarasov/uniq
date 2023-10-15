use super::token::Token;

fn read_byte<R>(read: &mut R) -> std::io::Result<Option<u8>>
where
    R: std::io::Read,
{
    let mut buf = [0u8; 1];
    match read.read_exact(&mut buf) {
        Ok(_) => Ok(Some(buf[0])),
        Err(error) => match error.kind() {
            std::io::ErrorKind::UnexpectedEof => Ok(None),
            _ => Err(error),
        },
    }
}

pub struct Lexer<'a, R> {
    read: &'a mut R,
    current: Option<u8>,
    offset: usize,
}

impl<'a, R> Lexer<'a, R>
where
    R: std::io::Read,
{
    pub fn new(read: &'a mut R) -> std::io::Result<Self> {
        Ok(Self {
            current: read_byte(read)?,
            offset: 0,
            read,
        })
    }

    fn advance(&mut self) -> std::io::Result<()> {
        self.current = read_byte(self.read)?;
        if self.current.is_some() {
            self.offset += 1;
        }
        Ok(())
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

    fn double_exclamation(&mut self) -> std::io::Result<Token> {
        self.advance()?;
        if let Some(c) = self.current {
            match c {
                b'=' => self.single(Token::ExclamationEqual),
                _ => Ok(Token::Exclamation)
            }
        } else {
            Ok(Token::Exclamation)
        }
    }

    fn double_less(&mut self) -> std::io::Result<Token> {
        self.advance()?;
        if let Some(c) = self.current {
            match c {
                b'=' => self.single(Token::LessEqual),
                _ => Ok(Token::Less)
            }
        } else {
            Ok(Token::Less)
        }
    }

    fn double_greater(&mut self) -> std::io::Result<Token> {
        self.advance()?;
        if let Some(c) = self.current {
            match c {
                b'=' => self.single(Token::GreaterEqual),
                _ => Ok(Token::Greater)
            }
        } else {
            Ok(Token::Greater)
        }
    }

    fn identifier(&mut self, buf: &mut Vec<u8>) -> std::io::Result<Token> {
        buf.clear();
        while let Some(c) = self.current {
            if c.is_ascii_alphanumeric() || c == b'_' {
                buf.push(c);
                self.advance()?;
            } else {
                break;
            }
        }
        Ok(match buf.as_slice() {
            b"true" => Token::True,
            b"false" => Token::False,
            b"if" => Token::If,
            b"else" => Token::Else,
            b"let" => Token::Let,
            b"while" => Token::While,
            b"for" => Token::For,
            b"return" => Token::Return,
            _ => Token::Identifier,
        })
    }

    pub fn lex(&mut self, buf: &mut Vec<u8>) -> std::io::Result<Token> {
        if let Some(c) = self.current {
            match c {
                b'+' => self.single(Token::Plus),
                b'-' => self.single(Token::Minus),
                b'*' => self.single(Token::Asterisk),
                b'/' => self.single(Token::Slash),
                b'(' => self.single(Token::LeftParen),
                b')' => self.single(Token::RightParen),
                b'{' => self.single(Token::LeftBrace),
                b'}' => self.single(Token::RightBrace),
                b'[' => self.single(Token::LeftBracket),
                b']' => self.single(Token::RightBracket),
                b',' => self.single(Token::Comma),
                b'|' => self.single(Token::VerticalBar),
                b';' => self.single(Token::Semicolon),
                b'!' => self.double_exclamation(),
                b'=' => self.double_equal(),
                b'<' => self.double_less(),
                b'>' => self.double_greater(),
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.identifier(buf),
                b'0'..=b'9' => self.integer(buf),
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
