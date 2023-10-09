use std::{collections::HashMap, ops::Range};

use crate::{lexer::Lexer, opcode, raise, CompRes, Token, CompilerError};

pub struct SrcPos {
    pub range: Range<usize>,
    pub source_id: usize,
}

pub struct Chunk {
    pub opcodes: Box<[u8]>,
    pub ranges: HashMap<usize, SrcPos>,
}

pub struct Compiler {
    token: Token,
    range: Range<usize>,
    opcodes: Vec<u8>,
    ranges: HashMap<usize, SrcPos>,
    buffer: Vec<u8>,
    source_id: usize,
}

fn get_precedence(token: Token) -> u8 {
    match token {
        Token::EqualEqual
        | Token::ExclamationEqual
        | Token::Greater
        | Token::Less
        | Token::LessEqual
        | Token::GreaterEqual => 2,
        Token::Plus | Token::Minus => 3,
        Token::Asterisk | Token::Slash => 4,
        _ => 0,
    }
}

impl Compiler {
    pub fn new(source_id: usize) -> Self {
        Self {
            token: Token::End,
            range: 0..0,
            opcodes: Vec::new(),
            ranges: HashMap::new(),
            buffer: Vec::new(),
            source_id,
        }
    }

    fn lex<R>(&mut self, lexer: &mut Lexer<R>) -> CompRes
    where
        R: std::io::Read,
    {
        lexer.skip_whitespaces()?;
        let start = lexer.offset();
        self.token = lexer.lex(&mut self.buffer)?;
        let end = lexer.offset();
        self.range = start..end;
        Ok(())
    }

    fn expect(&self, token: Token) -> CompRes {
        if self.token == token {
            Ok(())
        } else {
            Err(CompilerError::Custom(Box::new(format!("Expected {token}, found {}", self.token))))
        }
    }

    fn integer<R>(&mut self, lexer: &mut Lexer<R>) -> CompRes
    where
        R: std::io::Read,
    {
        let value = std::str::from_utf8(&self.buffer)?.parse::<u64>()?;
        if value <= 0xFF {
            self.opcodes.push(opcode::INT1);
            self.opcodes.push(value as u8);
        } else if value <= 0xFFFF {
            self.opcodes.push(opcode::INT2);
            self.opcodes.extend((value as u16).to_be_bytes());
        } else {
            self.opcodes.push(opcode::INT8);
            self.opcodes.extend(value.to_be_bytes());
        }
        self.lex(lexer) // Skip value.
    }

    fn real<R>(&mut self, lexer: &mut Lexer<R>) -> CompRes
    where
        R: std::io::Read,
    {
        let value = std::str::from_utf8(&self.buffer)?.parse::<f64>()?;
        self.opcodes.push(opcode::REAL);
        self.opcodes.extend(value.to_be_bytes());
        self.lex(lexer) // Skip value.
    }

    fn boolean<R>(&mut self, lexer: &mut Lexer<R>, value: bool) -> CompRes
    where
        R: std::io::Read,
    {
        self.opcodes.push(if value { opcode::TRUE } else { opcode::FALSE });
        self.lex(lexer) // Skip value.
    }

    fn subexpression<R>(&mut self, lexer: &mut Lexer<R>) -> CompRes
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '('.
        self.expression(lexer)?;
        self.expect(Token::RightParen)?;
        self.lex(lexer) // Skip ')'.
    }

    fn primary<R>(&mut self, lexer: &mut Lexer<R>) -> CompRes
    where
        R: std::io::Read,
    {
        match self.token {
            Token::Integer => self.integer(lexer),
            Token::Real => self.real(lexer),
            Token::True => self.boolean(lexer, true),
            Token::False => self.boolean(lexer, false),
            Token::LeftParen => self.subexpression(lexer),
            Token::Unknown => raise!("Unknown token."),
            Token::End => raise!("Unexpected end."),
            _ => raise!("Unexpected token."),
        }
    }

    fn binary<R>(&mut self, lexer: &mut Lexer<R>, precedence: u8) -> CompRes
    where
        R: std::io::Read,
    {
        loop {
            let current = get_precedence(self.token);

            if current < precedence {
                break Ok(());
            }

            let opcode = match self.token {
                Token::Plus => opcode::ADD,
                Token::Minus => opcode::SUB,
                Token::Asterisk => opcode::MUL,
                Token::Slash => opcode::DIV,
                Token::EqualEqual => opcode::EQ,
                Token::ExclamationEqual => opcode::NE,
                Token::Less => opcode::LS,
                Token::Greater => opcode::GR,
                Token::LessEqual => opcode::LE,
                Token::GreaterEqual => opcode::GE,
                _ => unreachable!(),
            };

            let range = self.range.clone();

            self.lex(lexer)?;
            self.primary(lexer)?;

            let next = get_precedence(self.token);

            if current < next {
                self.binary(lexer, current + 1)?;
            }

            self.ranges.insert(
                self.opcodes.len(),
                SrcPos {
                    range,
                    source_id: self.source_id,
                },
            );

            self.opcodes.push(opcode);
        }
    }

    fn expression<R>(&mut self, lexer: &mut Lexer<R>) -> CompRes
    where
        R: std::io::Read,
    {
        self.primary(lexer)?;
        self.binary(lexer, 1)
    }

    pub fn compile<R>(&mut self, source_id: usize, read: &mut R) -> CompRes
    where
        R: std::io::Read,
    {
        self.source_id = source_id;
        let mut lexer = Lexer::new(read)?;
        self.lex(&mut lexer)?;
        self.expression(&mut lexer)?;
        self.opcodes.push(opcode::RET);
        Ok(())
    }

    pub fn into_chunk(self) -> Chunk {
        Chunk {
            opcodes: self.opcodes.into_boxed_slice(),
            ranges: self.ranges,
        }
    }

    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    pub fn source_id(&self) -> usize {
        self.source_id
    }
}
