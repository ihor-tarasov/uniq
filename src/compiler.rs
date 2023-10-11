use std::{collections::HashMap, ops::Range};

use crate::{lexer::Lexer, opcode, raise, CompRes, CompilerError, Token};

pub struct SrcPos {
    pub range: Range<usize>,
    pub source_id: usize,
}

pub struct Chunk {
    pub opcodes: Box<[u8]>,
    pub ranges: HashMap<u32, SrcPos>,
}

pub struct Opcodes(Vec<u8>);

impl Opcodes {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn push(&mut self, opcode: u8) -> CompRes<()> {
        if self.0.len() <= u32::MAX as usize {
            self.0.push(opcode);
            Ok(())
        } else {
            Err(CompilerError::Custom(Box::new(format!("Too large opcodes count."))))
        }
    }

    fn extend<I>(&mut self, iter: I) -> CompRes<()>
    where
        I: IntoIterator<Item = u8>
    {
        for opcode in iter {
            self.push(opcode)?;
        }
        Ok(())
    }

    fn len(&self) -> u32 {
        self.0.len() as u32
    }
}

impl std::ops::Index<u32> for Opcodes {
    type Output = u8;

    fn index(&self, index: u32) -> &Self::Output {
        self.0.index(index as usize)
    }
}

impl std::ops::IndexMut<u32> for Opcodes {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        self.0.index_mut(index as usize)
    }
}

pub struct Compiler {
    token: Token,
    range: Range<usize>,
    opcodes: Opcodes,
    ranges: HashMap<u32, SrcPos>,
    buffer: Vec<u8>,
    address_stack: Vec<u32>,
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
            opcodes: Opcodes::new(),
            ranges: HashMap::new(),
            buffer: Vec::new(),
            address_stack: Vec::new(),
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
            Err(CompilerError::Custom(Box::new(format!(
                "Expected {token}, found {}",
                self.token
            ))))
        }
    }

    fn set_jf(&mut self, address: u32, value: u32) {
        if value <= 0xFFFF {
            self.opcodes[address] = opcode::JF2;
            (value as u16)
                .to_be_bytes()
                .iter()
                .cloned()
                .enumerate()
                .for_each(|(i, b)| self.opcodes[address + i as u32 + 1] = b);
        } else {
            self.opcodes[address] = opcode::JF4;
            value
                .to_be_bytes()
                .iter()
                .cloned()
                .enumerate()
                .for_each(|(i, b)| self.opcodes[address + i as u32 + 1] = b);
        }
    }

    fn set_jp(&mut self, address: u32, value: u32) {
        if value <= 0xFFFF {
            self.opcodes[address] = opcode::JP2;
            (value as u16)
                .to_be_bytes()
                .iter()
                .cloned()
                .enumerate()
                .for_each(|(i, b)| self.opcodes[address + i as u32 + 1] = b);
        } else {
            self.opcodes[address] = opcode::JP4;
            value
                .to_be_bytes()
                .iter()
                .cloned()
                .enumerate()
                .for_each(|(i, b)| self.opcodes[address + i as u32 + 1] = b);
        }
    }

    fn integer<R>(&mut self, lexer: &mut Lexer<R>) -> CompRes
    where
        R: std::io::Read,
    {
        let value = std::str::from_utf8(&self.buffer)?.parse::<u64>()?;
        if value <= 0xFF {
            self.opcodes.push(opcode::INT1)?;
            self.opcodes.push(value as u8)?;
        } else if value <= 0xFFFF {
            self.opcodes.push(opcode::INT2)?;
            self.opcodes.extend((value as u16).to_be_bytes())?;
        } else {
            self.opcodes.push(opcode::INT8)?;
            self.opcodes.extend(value.to_be_bytes())?;
        }
        self.lex(lexer) // Skip value.
    }

    fn real<R>(&mut self, lexer: &mut Lexer<R>) -> CompRes
    where
        R: std::io::Read,
    {
        let value = std::str::from_utf8(&self.buffer)?.parse::<f64>()?;
        self.opcodes.push(opcode::REAL)?;
        self.opcodes.extend(value.to_be_bytes())?;
        self.lex(lexer) // Skip value.
    }

    fn boolean<R>(&mut self, lexer: &mut Lexer<R>, value: bool) -> CompRes
    where
        R: std::io::Read,
    {
        self.opcodes
            .push(if value { opcode::TRUE } else { opcode::FALSE })?;
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

    fn statement<R>(&mut self, lexer: &mut Lexer<R>) -> CompRes
    where
        R: std::io::Read,
    {
        self.expression(lexer)
    }

    fn statements<R>(&mut self, lexer: &mut Lexer<R>, until: Token) -> CompRes
    where
        R: std::io::Read,
    {
        let mut first_time = true;
        loop {
            if first_time {
                first_time = false;
            } else {
                self.opcodes.push(opcode::DROP)?;
            }
            self.statement(lexer)?;

            if self.token == Token::Semicolon {
                self.lex(lexer)?; // Skip ';'
            }

            if self.token == until {
                break Ok(());
            }
        }
    }

    fn block<R>(&mut self, lexer: &mut Lexer<R>) -> CompRes
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '{'.
        self.statements(lexer, Token::RightBrace)?;
        self.lex(lexer) // Skip '}'.
    }

    fn if_stat<R>(&mut self, lexer: &mut Lexer<R>) -> CompRes
    where
        R: std::io::Read,
    {
        let mut address_stack_size: u32 = 0;
        loop {
            self.lex(lexer)?; // Skip 'if'.
            self.expression(lexer)?; // Condition.
            
            // JF to the next block.
            let next_jf_address = self.opcodes.len();
            self.opcodes.extend([0; 5])?;

            self.expect(Token::LeftBrace)?;

            self.block(lexer)?;

            if self.token == Token::Else {
                self.lex(lexer)?; // Skip 'else'.

                if self.token == Token::LeftBrace {
                    self.address_stack.push(self.opcodes.len());
                    address_stack_size += 1;
                    self.opcodes.extend([0; 5])?;
                    self.set_jf(next_jf_address, self.opcodes.len());
                    self.block(lexer)?;
                    break;
                } else if self.token == Token::If {
                    self.address_stack.push(self.opcodes.len());
                    address_stack_size += 1;
                    self.opcodes.extend([0; 5])?;
                    self.set_jf(next_jf_address, self.opcodes.len());
                }
            } else {
                self.address_stack.push(self.opcodes.len());
                address_stack_size += 1;
                self.opcodes.extend([0; 5])?;
                self.set_jf(next_jf_address, self.opcodes.len());
                self.opcodes.push(opcode::VOID)?;
                break;
            }
        }

        for _ in 0..address_stack_size {
            let jp_address = self.address_stack.pop().unwrap();
            self.set_jp(jp_address, self.opcodes.len());
        }

        Ok(())
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
            Token::LeftBrace => self.block(lexer),
            Token::If => self.if_stat(lexer),
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

            self.opcodes.push(opcode)?;
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
        self.expect(Token::End)?;
        self.opcodes.push(opcode::RET)?;
        Ok(())
    }

    pub fn into_chunk(self) -> Chunk {
        Chunk {
            opcodes: self.opcodes.0.into_boxed_slice(),
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
