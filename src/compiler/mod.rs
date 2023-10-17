mod block;
mod chunk;
mod cycles;
mod error;
mod expression;
mod function;
mod lexer;
mod opcodes;
mod pos;
mod statements;
mod token;
mod utils;
mod logic;

pub use chunk::*;
pub use error::*;
pub use pos::*;

use self::{block::Block, cycles::Cycles, function::Function, lexer::Lexer, token::Token};
use crate::{natives::Natives, opcode, raise};
use std::{collections::HashMap, ops::Range};

pub struct Compiler<'a> {
    token: Token,
    range: Range<usize>,
    chunk: Chunk,
    buffer: Vec<u8>,
    address_stack: Vec<u32>,
    source_id: usize,
    function: Function,
    globals: Block,
    natives: &'a Natives,
    cycles: Cycles,
    function_addresses: HashMap<Box<[u8]>, u32>,
}

impl<'a> Compiler<'a> {
    pub fn new(source_id: usize, natives: &'a Natives) -> Self {
        Self {
            token: Token::End,
            range: 0..0,
            chunk: Chunk::new(),
            buffer: Vec::new(),
            address_stack: Vec::new(),
            source_id,
            function: Function::new(),
            globals: Block::new(),
            natives,
            cycles: Cycles::new(),
            function_addresses: HashMap::new(),
        }
    }

    fn lex<R>(&mut self, lexer: &mut Lexer<R>) -> Res
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

    fn expect(&self, token: Token) -> Res {
        if self.token == token {
            Ok(())
        } else {
            raise!("Expected {token}, found {}", self.token)
        }
    }

    fn block<R>(&mut self, lexer: &mut Lexer<R>) -> Res
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

    fn find_function(&self, name: &[u8]) -> Option<u32> {
        self.function_addresses.get(name).cloned()
    }

    fn add_local(&mut self) -> u32 {
        self.function.var(&self.buffer)
    }

    fn add_global(&mut self) -> u32 {
        if let Some(id) = self.globals.get(&self.buffer) {
            id
        } else {
            let id = self.globals.len();
            assert!(id < (u32::MAX as usize));
            self.globals.var(&self.buffer, id as u32);
            id as u32
        }
    }

    fn add_function(&mut self, address: u32) -> Res {
        if self
            .function_addresses
            .insert(
                self.buffer.as_slice().to_owned().into_boxed_slice(),
                address,
            )
            .is_some()
        {
            raise!(
                "Function \"{}\" already exists.",
                utils::Slice(&self.buffer)
            )
        } else {
            Ok(())
        }
    }

    fn enter_block(&mut self) {
        self.function.push();
    }

    fn exit_block(&mut self) {
        self.function.pop();
    }

    fn function<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip 'fn'.

        let end_jp_address = self.chunk.empty_address()?;

        self.expect(Token::Identifier)?;
        self.add_function(self.chunk.len())?;
        self.lex(lexer)?; // Skip function name.
        self.expect(Token::LeftParen)?;
        self.lex(lexer)?; // Skip '('.

        let mut args_count = 0;
        while self.token == Token::Identifier {
            if args_count > u8::MAX as u32 {
                return raise!("Reached maximum function argumens number.");
            }
            self.add_local();
            self.lex(lexer)?; // Skip argument name.
            args_count += 1;

            if self.token == Token::Comma {
                self.lex(lexer)?; // Skip ','.
            }
        }

        self.expect(Token::RightParen)?;
        self.lex(lexer)?; // Skip ')'.

        let stack_size_address = self.chunk.start_function(args_count as u8)?;

        self.expect(Token::LeftBrace)?;
        self.block(lexer)?;

        self.chunk.push(opcode::RET)?;

        self.chunk
            .write_u32_at(stack_size_address, self.function.stack_size() - args_count);

        self.function.clear();

        self.chunk.set_jp(end_jp_address, self.chunk.len());

        Ok(())
    }

    pub fn compile<R>(&mut self, source_id: usize, read: &mut R) -> Res
    where
        R: std::io::Read,
    {
        if self.chunk.len() != 0 {
            self.chunk.pop();
        }

        self.source_id = source_id;
        let mut lexer = Lexer::new(read)?;
        self.lex(&mut lexer)?;

        loop {
            match self.token {
                Token::End => break,
                Token::Fn => self.function(&mut lexer)?,
                Token::Let => {
                    self.let_stat(&mut lexer, true, true)?;
                    self.chunk.push(opcode::DROP)?;
                }
                _ => {
                    self.expression(&mut lexer, true)?;
                    self.expect(Token::Semicolon)?;
                    self.lex(&mut lexer)?; // Skip ';'.
                    self.chunk.push(opcode::DROP)?;
                }
            }
        }

        self.chunk.push(opcode::RET)?;

        Ok(())
    }

    pub fn chunk(&self) -> &Chunk {
        &self.chunk
    }

    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    pub fn source_id(&self) -> usize {
        self.source_id
    }

    pub fn into_chunk(self) -> Chunk {
        self.chunk
    }
}
