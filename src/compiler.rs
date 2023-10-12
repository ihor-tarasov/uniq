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
            Err(CompilerError::Custom(Box::new(format!(
                "Too large opcodes count."
            ))))
        }
    }

    fn extend<I>(&mut self, iter: I) -> CompRes<()>
    where
        I: IntoIterator<Item = u8>,
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

struct Block {
    locals: HashMap<Box<[u8]>, u32>,
}

impl Block {
    fn new() -> Self {
        Self {
            locals: HashMap::new(),
        }
    }

    fn var(&mut self, name: &[u8], id: u32) -> bool {
        if let Some(local) = self.locals.get_mut(name) {
            *local = id;
            true
        } else {
            self.locals.insert(Vec::from(name).into_boxed_slice(), id);
            false
        }
    }

    fn get(&self, name: &[u8]) -> Option<u32> {
        self.locals.get(name).cloned()
    }

    fn len(&self) -> usize {
        self.locals.len()
    }
}

struct Function {
    blocks: Vec<Block>,
    local_counter: u32,
    stack_size: u32,
}

impl Function {
    fn new() -> Self {
        Self {
            blocks: vec![Block::new()],
            local_counter: 0,
            stack_size: 0,
        }
    }

    fn push(&mut self) {
        self.blocks.push(Block::new());
    }

    fn pop(&mut self) {
        let block = self.blocks.pop().unwrap();
        self.local_counter -= block.len() as u32;
    }

    fn var(&mut self, name: &[u8]) -> u32 {
        debug_assert!(!self.blocks.is_empty());
        let len = self.blocks.len();
        let id = self.local_counter;
        if !self.blocks[len - 1].var(name, id) {
            self.local_counter += 1;
            if self.local_counter > self.stack_size {
                self.stack_size = self.local_counter;
            }
        }
        id
    }

    fn get(&self, name: &[u8]) -> Option<u32> {
        for block in self.blocks.iter().rev() {
            if let Some(id) = block.get(name) {
                return Some(id);
            }
        }
        None
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
    functions: Vec<Function>,
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
            functions: Vec::new(),
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
        self.enter_block();
        self.lex(lexer)?; // Skip '{'.
        self.statements(lexer, Token::RightBrace)?;
        self.lex(lexer)?; // Skip '}'.
        self.exit_block();
        Ok(())
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

    fn find_variable(&self, name: &[u8]) -> Option<u32> {
        self.functions.last().unwrap().get(name)
    }

    fn call<R>(&mut self, lexer: &mut Lexer<R>) -> CompRes
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '('.

        let mut arguments = 0;
        loop {
            if arguments > u8::MAX as u32 {
                return Err(CompilerError::Custom(Box::new(format!(
                    "Reached maximum function argumens number."
                ))));
            }

            if self.token == Token::RightParen {
                break;
            }

            self.expression(lexer)?;
            arguments += 1;

            if self.token == Token::Comma {
                self.lex(lexer)?; // Skip ','
            }
        }

        self.lex(lexer)?; // Skip ')'.

        self.opcodes.extend([opcode::CALL, arguments as u8])
    }

    fn store(&mut self, index: u32) -> CompRes {
        if index <= u8::MAX as u32 {
            self.opcodes.extend([opcode::ST1, index as u8])
        } else if index <= 0xFFFF {
            self.opcodes.push(opcode::ST2)?;
            self.opcodes.extend((index as u16).to_be_bytes())
        } else {
            self.opcodes.push(opcode::ST4)?;
            self.opcodes.extend(index.to_le_bytes())
        }
    }

    fn assign<R>(&mut self, lexer: &mut Lexer<R>, index: u32) -> CompRes
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '='.
        self.expression(lexer)?;
        self.store(index)
    }

    fn load(&mut self, index: u32) -> CompRes {
        if index <= u8::MAX as u32 {
            self.opcodes.extend([opcode::LD1, index as u8])
        } else if index <= 0xFFFF {
            self.opcodes.push(opcode::LD2)?;
            self.opcodes.extend((index as u16).to_be_bytes())
        } else {
            self.opcodes.push(opcode::LD4)?;
            self.opcodes.extend(index.to_le_bytes())
        }
    }

    fn index<R>(&mut self, lexer: &mut Lexer<R>, index: u32) -> CompRes
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '['.
        self.load(index)?;
        self.expression(lexer)?;
        self.expect(Token::RightBracket)?;
        self.lex(lexer)?; // Skip ']'.
        match self.token {
            Token::Equal => {
                self.lex(lexer)?; // Skip '='.
                self.expression(lexer)?;
                self.opcodes.push(opcode::SET)
            }
            _ => self.opcodes.push(opcode::GET),
        }
    }

    fn identifier<R>(&mut self, lexer: &mut Lexer<R>) -> CompRes
    where
        R: std::io::Read,
    {
        if let Some(index) = self.find_variable(&self.buffer) {
            self.lex(lexer)?; // Skip identifier.
            match self.token {
                Token::Equal => self.assign(lexer, index),
                Token::LeftBracket => self.index(lexer, index),
                _ => self.load(index),
            }
        } else {
            Err(CompilerError::Custom(Box::new(format!(
                "Unknown identifier \"{}\".",
                std::str::from_utf8(&self.buffer)?
            ))))
        }
    }

    fn add_local(&mut self) -> u32 {
        self.functions.last_mut().unwrap().var(&self.buffer)
    }

    fn let_stat<R>(&mut self, lexer: &mut Lexer<R>) -> CompRes
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip 'let'.
        self.expect(Token::Identifier)?;
        let local_id = self.add_local();
        self.lex(lexer)?; // Skip vriable name.
        self.expect(Token::Equal)?;
        self.lex(lexer)?; // Skip '='.
        self.expression(lexer)?;
        self.store(local_id)
    }

    fn enter_block(&mut self) {
        self.functions.last_mut().unwrap().push();
    }

    fn exit_block(&mut self) {
        self.functions.last_mut().unwrap().pop();
    }

    fn jump(&mut self, address: u32) -> CompRes {
        if address <= 0xFFFF {
            self.opcodes.push(opcode::JP2)?;
            self.opcodes.extend((address as u16).to_be_bytes())?;
            self.opcodes.extend([0; 2])
        } else {
            self.opcodes.push(opcode::JP4)?;
            self.opcodes.extend(address.to_be_bytes())
        }
    }

    fn while_stat<R>(&mut self, lexer: &mut Lexer<R>) -> CompRes
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip 'while'.
        self.opcodes.push(opcode::VOID)?;
        let while_start = self.opcodes.len();
        // Condition.
        self.expression(lexer)?;
        let end_jf_address = self.opcodes.len();
        self.opcodes.extend([0; 5])?;
        self.opcodes.push(opcode::DROP)?;
        // Block.
        self.expect(Token::LeftBrace)?;
        self.block(lexer)?;
        self.jump(while_start)?;
        self.set_jf(end_jf_address, self.opcodes.len());
        Ok(())
    }

    fn list<R>(&mut self, lexer: &mut Lexer<R>) -> CompRes
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '['.
        self.opcodes.push(opcode::LIST)?;

        while self.token != Token::RightBracket {
            self.expression(lexer)?;
            self.opcodes.push(opcode::ADD)?;
            if self.token == Token::Comma {
                self.lex(lexer)?; // Skip ','.
            }
        }

        self.lex(lexer) // Skip ']'
    }

    fn function<R>(&mut self, lexer: &mut Lexer<R>) -> CompRes
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '|'.

        let end_jp_address = self.opcodes.len();
        self.opcodes.extend([0; 5])?;

        self.functions.push(Function::new());

        let mut args_count = 0;
        while self.token == Token::Identifier {
            if args_count > u8::MAX as u32 {
                return Err(CompilerError::Custom(Box::new(format!(
                    "Reached maximum function argumens number."
                ))));
            }
            self.add_local();
            self.lex(lexer)?; // Skip argument name.
            args_count += 1;

            if self.token == Token::Comma {
                self.lex(lexer)?; // Skip ','.
            }
        }

        self.expect(Token::VerticalBar)?;
        self.lex(lexer)?; // Skip '|'.

        let function_address = self.opcodes.len();

        self.opcodes.push(args_count as u8)?;
        let stack_size_address = self.opcodes.len();
        self.opcodes.extend([0; 4])?;

        self.expect(Token::LeftBrace)?;
        self.block(lexer)?;

        self.opcodes.push(opcode::RET)?;

        let function = self.functions.pop().unwrap();

        (function.stack_size - args_count)
            .to_be_bytes()
            .iter()
            .cloned()
            .enumerate()
            .for_each(|(i, b)| self.opcodes[stack_size_address + i as u32] = b);

        self.set_jp(end_jp_address, self.opcodes.len());

        self.opcodes.push(opcode::PTR)?;
        self.opcodes.extend(function_address.to_be_bytes())?;

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
            Token::Let => self.let_stat(lexer),
            Token::Identifier => self.identifier(lexer),
            Token::While => self.while_stat(lexer),
            Token::LeftBracket => self.list(lexer),
            Token::VerticalBar => self.function(lexer),
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
        self.binary(lexer, 1)?;

        if self.token == Token::LeftParen {
            self.call(lexer)?;
        }

        Ok(())
    }

    pub fn compile<R>(&mut self, source_id: usize, read: &mut R) -> CompRes
    where
        R: std::io::Read,
    {
        self.source_id = source_id;
        let mut lexer = Lexer::new(read)?;
        self.lex(&mut lexer)?;

        let stack_size_address = self.opcodes.len();
        self.opcodes.extend([0; 4])?;

        self.functions.push(Function::new());

        self.statements(&mut lexer, Token::End)?;
        
        self.opcodes.push(opcode::RET)?;
        let function = self.functions.pop().unwrap();

        function.stack_size
            .to_be_bytes()
            .iter()
            .cloned()
            .enumerate()
            .for_each(|(i, b)| self.opcodes[stack_size_address + i as u32] = b);

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
