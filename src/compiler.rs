use std::ops::Range;

use crate::{lexer::Lexer, opcode, Token, CompRes, raise};

pub struct Compiler {
    token: Token,
    range: Range<usize>,
    opcodes: Vec<u8>,
    // ranges: HashMap<usize, Range<usize>>,
    buffer: Vec<u8>,
}

fn get_precedence(token: Token) -> u8 {
    match token {
        Token::Plus | Token::Minus => 2,
        Token::Asterisk => 3,
        _ => 0,
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            token: Token::End,
            range: 0..0,
            opcodes: Vec::new(),
            // ranges: HashMap::new(),
            buffer: Vec::new(),
        }
    }

    fn lex<I>(&mut self, lexer: &mut Lexer<I>) -> CompRes
    where
        I: Iterator<Item = std::io::Result<u8>>,
    {
        lexer.skip_whitespaces()?;
        let start = lexer.offset();
        self.token = lexer.lex(&mut self.buffer)?;
        let end = lexer.offset();
        self.range = start..end;
        Ok(())
    }

    fn integer<I>(&mut self, lexer: &mut Lexer<I>) -> CompRes
    where
        I: Iterator<Item = std::io::Result<u8>>,
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
        self.lex(lexer) // Skip integer value.
    }

    fn primary<I>(&mut self, lexer: &mut Lexer<I>) -> CompRes
    where
        I: Iterator<Item = std::io::Result<u8>>,
    {
        match self.token {
            Token::Integer => self.integer(lexer),
            Token::Unknown => raise!("Unknown token."),
            Token::End => raise!("Unexpected end."),
            _ => raise!("Unexpected token."),
        }
    }

    fn binary<I>(&mut self, lexer: &mut Lexer<I>, precedence: u8) -> CompRes
    where
        I: Iterator<Item = std::io::Result<u8>>,
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
                _ => unreachable!(),
            };

            self.lex(lexer)?;
            self.primary(lexer)?;

            let next = get_precedence(self.token);

            if current < next {
                self.binary(lexer, current + 1)?;
            }

            self.opcodes.push(opcode);
        }
    }

    fn expression<I>(&mut self, lexer: &mut Lexer<I>) -> CompRes
    where
        I: Iterator<Item = std::io::Result<u8>>,
    {
        self.primary(lexer)?;
        self.binary(lexer, 1)
    }

    pub fn compile<I>(&mut self, iter: I) -> CompRes
    where
        I: Iterator<Item = std::io::Result<u8>>,
    {
        let mut lexer = Lexer::new(iter)?;
        self.lex(&mut lexer)?;
        self.expression(&mut lexer)?;
        self.opcodes.push(opcode::RET);
        Ok(())
    }

    pub fn finish(self) -> Vec<u8> {
        self.opcodes
    }
}
