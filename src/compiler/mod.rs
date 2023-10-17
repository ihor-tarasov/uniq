mod block;
mod chunk;
mod error;
mod function;
mod lexer;
mod opcodes;
mod pos;
mod token;

pub use chunk::*;
pub use error::*;
pub use pos::*;

use self::{function::Function, lexer::Lexer, token::Token};
use crate::{natives::Natives, opcode, raise};
use std::{collections::HashMap, ops::Range};

struct Slice<'a>(pub &'a [u8]);

impl<'a> std::fmt::Display for Slice<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in self.0 {
            write!(f, "{}", *c as char)?;
        }
        Ok(())
    }
}

pub struct Compiler<'a> {
    token: Token,
    range: Range<usize>,
    chunk: Chunk,
    buffer: Vec<u8>,
    address_stack: Vec<u32>,
    source_id: usize,
    function: Function,
    global: Function,
    is_parsing_function: bool,
    natives: &'a Natives,
    cycles_end_addresses: Vec<u32>,
    cycles_end_addresses_sizes: Vec<u32>,
    cycles_starts: Vec<u32>,
    function_addresses: HashMap<Box<[u8]>, u32>,
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
            global: Function::new(),
            is_parsing_function: false,
            natives,
            cycles_end_addresses: Vec::new(),
            cycles_end_addresses_sizes: Vec::new(),
            cycles_starts: Vec::new(),
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

    fn integer<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.chunk.integer(&self.buffer)?;
        self.lex(lexer) // Skip value.
    }

    fn real<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.chunk.real(&self.buffer)?;
        self.lex(lexer) // Skip value.
    }

    fn boolean<R>(&mut self, lexer: &mut Lexer<R>, value: bool) -> Res
    where
        R: std::io::Read,
    {
        self.chunk.boolean(value)?;
        self.lex(lexer) // Skip value.
    }

    fn subexpression<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '('.
        self.expression(lexer)?;
        self.expect(Token::RightParen)?;
        self.lex(lexer) // Skip ')'.
    }

    fn return_stat<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip 'return'.

        match self.token {
            Token::Semicolon => self.chunk.push(opcode::VOID)?,
            _ => {
                self.expression(lexer)?;
                self.expect(Token::Semicolon)?;
            }
        }

        self.lex(lexer)?; // Skip ';'.
        self.chunk.push(opcode::RET)
    }

    fn break_stat<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip 'break'.

        match self.token {
            Token::Semicolon => self.chunk.push(opcode::VOID)?,
            _ => {
                self.expression(lexer)?;
                self.expect(Token::Semicolon)?;
            }
        }
        
        self.lex(lexer)?; // Skip ';'.

        let address = self.chunk.empty_address()?;

        if let Some(cycle_address_size) = self.cycles_end_addresses_sizes.last_mut() {
            *cycle_address_size += 1;
            self.cycles_end_addresses.push(address);
            Ok(())
        } else {
            raise!("Unable to use 'break' statement in this place.")
        }
    }

    fn continue_stat<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip 'continue'.

        match self.token {
            Token::Semicolon => self.chunk.push(opcode::VOID)?,
            _ => {
                self.expression(lexer)?;
                self.expect(Token::Semicolon)?;
            }
        }

        self.lex(lexer)?; // Skip ';'.

        if let Some(cycle_start) = self.cycles_starts.last().cloned() {
            self.chunk.jump(cycle_start)
        } else {
            raise!("Unable to use 'continue' statement in this place.")
        }
    }

    fn statements<R>(&mut self, lexer: &mut Lexer<R>, until: Token, is_global: bool) -> Res
    where
        R: std::io::Read,
    {
        let mut first_time = true;
        let mut last_function = false;
        loop {
            if first_time {
                first_time = false;
            } else {
                if last_function {
                    last_function = false;
                } else {
                    self.chunk.push(opcode::DROP)?;
                }
            }

            if is_global && self.token == Token::Fn {
                last_function = true;
                self.function(lexer)?;
            } else {
                match self.token {
                    Token::Return => self.return_stat(lexer)?,
                    Token::Break => self.break_stat(lexer)?,
                    Token::Continue => self.continue_stat(lexer)?,
                    Token::If => self.if_stat(lexer)?,
                    Token::While => self.while_stat(lexer)?,
                    Token::For => self.for_stat(lexer)?,
                    Token::LeftBrace => self.block(lexer)?,
                    Token::Let => self.let_stat(lexer, true)?,
                    _ => {
                        self.expression(lexer)?;
                        self.expect(Token::Semicolon)?;
                        self.lex(lexer)?; // Skip ';'.
                    }
                }
            }

            if self.token == until {
                break Ok(());
            }
        }
    }

    fn block<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.enter_block();
        self.lex(lexer)?; // Skip '{'.
        self.statements(lexer, Token::RightBrace, false)?;
        self.lex(lexer)?; // Skip '}'.
        self.exit_block();
        Ok(())
    }

    fn if_stat<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        let mut address_stack_size: u32 = 0;
        loop {
            self.lex(lexer)?; // Skip 'if'.
            self.expression(lexer)?; // Condition.

            // JF to the next block.
            let next_jf_address = self.chunk.empty_address()?;

            self.expect(Token::LeftBrace)?;

            self.block(lexer)?;

            if self.token == Token::Else {
                self.lex(lexer)?; // Skip 'else'.

                if self.token == Token::LeftBrace {
                    self.address_stack.push(self.chunk.empty_address()?);
                    address_stack_size += 1;
                    self.chunk.set_jf(next_jf_address, self.chunk.len());
                    self.block(lexer)?;
                    break;
                } else if self.token == Token::If {
                    self.address_stack.push(self.chunk.empty_address()?);
                    address_stack_size += 1;
                    self.chunk.set_jf(next_jf_address, self.chunk.len());
                }
            } else {
                self.address_stack.push(self.chunk.empty_address()?);
                address_stack_size += 1;
                self.chunk.set_jf(next_jf_address, self.chunk.len());
                self.chunk.push(opcode::VOID)?;
                break;
            }
        }

        for _ in 0..address_stack_size {
            let jp_address = self.address_stack.pop().unwrap();
            self.chunk.set_jp(jp_address, self.chunk.len());
        }

        Ok(())
    }

    fn find_local(&self, name: &[u8]) -> Option<u32> {
        if self.is_parsing_function {
            self.function.get(name)
        } else {
            None
        }
    }

    fn find_global(&self, name: &[u8]) -> Option<u32> {
        self.global.get(name)
    }

    fn find_function(&self, name: &[u8]) -> Option<u32> {
        self.function_addresses.get(name).cloned()
    }

    fn call<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '('.

        let mut arguments = 0;
        loop {
            if arguments > u8::MAX as u32 {
                return raise!("Reached maximum function argumens number.");
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

        self.chunk.call(arguments as u8)
    }

    fn assign<R>(&mut self, lexer: &mut Lexer<R>, index: u32, is_local: bool) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '='.
        self.expression(lexer)?;
        self.chunk.store(index, is_local)
    }

    fn index<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '['.
        self.expression(lexer)?;
        self.expect(Token::RightBracket)?;
        self.lex(lexer)?; // Skip ']'.
        match self.token {
            Token::Equal => {
                self.lex(lexer)?; // Skip '='.
                self.expression(lexer)?;
                self.chunk.push(opcode::SET)
            }
            _ => self.chunk.push(opcode::GET),
        }
    }

    fn postfix_identifier_increment<R>(
        &mut self,
        lexer: &mut Lexer<R>,
        index: u32,
        is_local: bool,
    ) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '++'.
        self.chunk.load(index, is_local)?;
        self.chunk.push(opcode::INC)?;
        self.chunk.store(index, is_local)
    }

    fn postfix_identifier_decrement<R>(
        &mut self,
        lexer: &mut Lexer<R>,
        index: u32,
        is_local: bool,
    ) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '--'.
        self.chunk.load(index, is_local)?;
        self.chunk.push(opcode::DEC)?;
        self.chunk.store(index, is_local)
    }

    fn identifier<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        if let Some(index) = self.find_local(&self.buffer) {
            self.lex(lexer)?; // Skip identifier.
            match self.token {
                Token::Equal => self.assign(lexer, index, true),
                Token::PlusPlus => self.postfix_identifier_increment(lexer, index, true),
                Token::MinusMinus => self.postfix_identifier_decrement(lexer, index, true),
                _ => self.chunk.load(index, true),
            }
        } else if let Some(index) = self.find_global(&self.buffer) {
            self.lex(lexer)?; // Skip identifier.
            match self.token {
                Token::Equal => self.assign(lexer, index, false),
                Token::PlusPlus => self.postfix_identifier_increment(lexer, index, false),
                Token::MinusMinus => self.postfix_identifier_decrement(lexer, index, false),
                _ => self.chunk.load(index, false),
            }
        } else if let Some(address) = self.find_function(&self.buffer) {
            self.lex(lexer)?; // Skip identifier.
            self.chunk.ptr(address)
        } else if let Some(index) = self.natives.get_index(&self.buffer) {
            self.lex(lexer)?; // Skip identifier.
            self.chunk.nat(index)
        } else {
            raise!(
                "Unknown identifier \"{}\".",
                std::str::from_utf8(&self.buffer)?
            )
        }
    }

    fn add_local(&mut self) -> u32 {
        if self.is_parsing_function {
            self.function.var(&self.buffer)
        } else {
            self.global.var(&self.buffer)
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
            raise!("Function \"{}\" already exists.", Slice(&self.buffer))
        } else {
            Ok(())
        }
    }

    fn let_stat<R>(&mut self, lexer: &mut Lexer<R>, expect_semicolon: bool) -> Res
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
        self.chunk.store(local_id, self.is_parsing_function)?;
        if expect_semicolon {
            self.expect(Token::Semicolon)?;
            self.lex(lexer) // Skip ';'.
        } else {
            Ok(())
        }
    }

    fn enter_block(&mut self) {
        if self.is_parsing_function {
            self.function.push();
        } else {
            self.global.push();
        }
    }

    fn exit_block(&mut self) {
        if self.is_parsing_function {
            self.function.pop();
        } else {
            self.global.pop();
        }
    }

    /*

    2 == 0 and 3 == 2

        INT 2
        INT 0
        EQ

        JF end_false

        INT 3
        INT 2
        EQ

        JF end_false

        TRUE
        JP end

    end_false:
        FALSE
    end:

    */

    fn logic_and<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip 'and'.

        let mut address_count = 0;

        let end_false_address = self.chunk.empty_address()?;
        self.address_stack.push(end_false_address);
        address_count += 1;

        loop {
            self.expression_without_logic(lexer)?;

            let end_false_address = self.chunk.empty_address()?;
            self.address_stack.push(end_false_address);
            address_count += 1;

            match self.token {
                Token::And => {
                    self.lex(lexer)?; // Skip 'and'.
                }
                Token::Or => {
                    return raise!("Unable to combine 'and' and 'or' operators.");
                }
                _ => {
                    break;
                }
            }
        }

        self.chunk.boolean(true)?;

        let end_address = self.chunk.empty_address()?;

        for _ in 0..address_count {
            let address = self.address_stack.pop().unwrap();
            self.chunk.set_jf(address, self.chunk.len());
        }

        self.chunk.boolean(false)?;

        self.chunk.set_jp(end_address, self.chunk.len());

        Ok(())
    }

    /*

    2 == 0 or 3 == 2

        INT 2
        INT 0
        EQ

        JT end_true

        INT 3
        INT 2
        EQ

        JT end_true

        FALSE
        JP end

    end_true:
        TRUE
    end:

    */

    fn logic_or<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip 'and'.

        let mut address_count = 0;

        let end_true_address = self.chunk.empty_address()?;
        self.address_stack.push(end_true_address);
        address_count += 1;

        loop {
            self.expression_without_logic(lexer)?;

            let end_true_address = self.chunk.empty_address()?;
            self.address_stack.push(end_true_address);
            address_count += 1;

            match self.token {
                Token::Or => {
                    self.lex(lexer)?; // Skip 'or'.
                }
                Token::And => {
                    return raise!("Unable to combine 'and' and 'or' operators.");
                }
                _ => {
                    break;
                }
            }
        }

        self.chunk.boolean(false)?;

        let end_address = self.chunk.empty_address()?;

        for _ in 0..address_count {
            let address = self.address_stack.pop().unwrap();
            self.chunk.set_jt(address, self.chunk.len());
        }

        self.chunk.boolean(true)?;

        self.chunk.set_jp(end_address, self.chunk.len());

        Ok(())
    }

    fn while_stat<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip 'while'.
        self.chunk.push(opcode::VOID)?;
        let while_start = self.chunk.len();
        // Condition.
        self.expression(lexer)?;
        let end_jf_address = self.chunk.empty_address()?;
        self.chunk.push(opcode::DROP)?;

        // Block.
        self.cycles_starts.push(while_start);
        self.cycles_end_addresses_sizes.push(0);

        self.expect(Token::LeftBrace)?;
        self.block(lexer)?;
        self.chunk.jump(while_start)?;

        self.cycles_starts.pop().unwrap();
        let ends_size = self.cycles_end_addresses_sizes.pop().unwrap();

        for _ in 0..ends_size {
            let address = self.cycles_end_addresses.pop().unwrap();
            self.chunk.set_jp(address, self.chunk.len());
        }

        self.chunk.set_jf(end_jf_address, self.chunk.len());
        Ok(())
    }

    /*

    for i = 0, i < 10, i = i + 1 {
        print(i);
    }

        ; i = 0
        INT 0
        ST i
        DROP

        VOID ; If 0 iterations
    start:
        ; i < 10
        INT 10
        LD i
        LS
        JF end
        DROP ; Drop result of the last iteration

        JP skip
    step:
        ; i = i + 1
        INT 1
        LD i
        ADD
        ST i
        DROP
        JP end_step
    skip:
        ; print(i)
        NAT print
        LD i
        CALL 1

        JP step
    end_step:
        JP for_start
    end:
    */

    fn for_stat<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip 'for'.

        self.enter_block();

        let local_id = self.add_local();
        self.lex(lexer)?; // Skip variable name.

        self.expect(Token::Equal)?;
        self.lex(lexer)?; // Skip '='.

        self.expression(lexer)?;
        self.chunk.store(local_id, true)?;
        self.chunk.push(opcode::DROP)?;

        self.expect(Token::Comma)?;
        self.lex(lexer)?; // Skip ','.

        self.chunk.push(opcode::VOID)?;
        let start = self.chunk.len();

        // Condition.
        self.expression(lexer)?;
        let end_address = self.chunk.empty_address()?;
        self.chunk.push(opcode::DROP)?;

        let skip_address = self.chunk.empty_address()?;
        let step = self.chunk.len();

        self.expect(Token::Comma)?;
        self.lex(lexer)?; // Skip ','.

        self.expression(lexer)?;
        self.chunk.push(opcode::DROP)?;

        let end_step_address = self.chunk.empty_address()?;

        self.chunk.set_jp(skip_address, self.chunk.len());

        // Block.
        self.expect(Token::LeftBrace)?;
        self.block(lexer)?;

        self.chunk.jump(step)?;
        self.chunk.set_jp(end_step_address, self.chunk.len());

        self.chunk.jump(start)?;

        self.chunk.set_jf(end_address, self.chunk.len());

        self.exit_block();

        Ok(())
    }

    fn list<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '['.
        self.chunk.push(opcode::LIST)?;

        while self.token != Token::RightBracket {
            self.expression(lexer)?;
            self.chunk.push(opcode::ADD)?;
            match self.token {
                Token::RightBracket => break,
                Token::Comma => {
                    self.lex(lexer)?; // Skip ','.
                }
                _ => return raise!("Expected ',' or ']', found {}", self.token),
            }
        }

        self.lex(lexer) // Skip ']'
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

        self.is_parsing_function = true;

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

        self.is_parsing_function = false;

        self.chunk.write_u32_at(stack_size_address, self.function.stack_size() - args_count);

        self.function.clear();

        self.chunk.set_jp(end_jp_address, self.chunk.len());

        Ok(())
    }

    fn primary<R>(&mut self, lexer: &mut Lexer<R>) -> Res
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
            Token::Let => self.let_stat(lexer, false),
            Token::Identifier => self.identifier(lexer),
            Token::While => self.while_stat(lexer),
            Token::LeftBracket => self.list(lexer),
            Token::For => self.for_stat(lexer),
            Token::Unknown => raise!("Unknown token."),
            Token::End => raise!("Unexpected end."),
            _ => raise!("Unexpected token."),
        }
    }

    fn secondary<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.primary(lexer)?;

        loop {
            match self.token {
                Token::LeftParen => self.call(lexer)?,
                Token::LeftBracket => self.index(lexer)?,
                _ => break Ok(()),
            }
        }
    }

    fn unary<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        match self.token {
            Token::Exclamation => {
                self.lex(lexer)?; // Skip '!'.
                self.secondary(lexer)?;
                self.chunk.push(opcode::NOT)
            }
            Token::Minus => {
                self.lex(lexer)?; // Skip '-'.
                self.secondary(lexer)?;
                self.chunk.push(opcode::NEG)
            }
            _ => self.secondary(lexer),
        }
    }

    fn binary<R>(&mut self, lexer: &mut Lexer<R>, precedence: u8) -> Res
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
            self.unary(lexer)?;

            let next = get_precedence(self.token);

            if current < next {
                self.binary(lexer, current + 1)?;
            }

            self.chunk.push_pos(Pos {
                range,
                source_id: self.source_id,
            });

            self.chunk.push(opcode)?;
        }
    }

    fn expression_without_logic<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.unary(lexer)?;
        self.binary(lexer, 1)
    }

    fn expression<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.expression_without_logic(lexer)?;

        match self.token {
            Token::And => self.logic_and(lexer),
            Token::Or => self.logic_or(lexer),
            _ => Ok(()),
        }
    }

    pub fn compile<R>(&mut self, source_id: usize, read: &mut R) -> Res
    where
        R: std::io::Read,
    {
        if self.chunk.len() == 0 {
            self.chunk.start_global()?;
        } else {
            self.chunk.pop();
        }

        self.source_id = source_id;
        let mut lexer = Lexer::new(read)?;
        self.lex(&mut lexer)?;

        self.is_parsing_function = false;

        self.statements(&mut lexer, Token::End, true)?;

        self.chunk.push(opcode::RET)?;

        Ok(())
    }

    pub fn finish(&mut self) {
        self.chunk.write_u32_at(0, self.global.stack_size());
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

    pub fn len(&self) -> u32 {
        self.chunk.len()
    }
}
