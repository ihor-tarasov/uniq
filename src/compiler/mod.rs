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

use self::{function::Function, lexer::Lexer, opcodes::Opcodes, token::Token};
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
    opcodes: Opcodes,
    ranges: HashMap<u32, Pos>,
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
            opcodes: Opcodes::new(),
            ranges: HashMap::new(),
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

    fn set_jt(&mut self, address: u32, value: u32) {
        if value <= 0xFFFF {
            self.opcodes[address] = opcode::JT2;
            (value as u16)
                .to_be_bytes()
                .iter()
                .cloned()
                .enumerate()
                .for_each(|(i, b)| self.opcodes[address + i as u32 + 1] = b);
        } else {
            self.opcodes[address] = opcode::JT4;
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

    fn integer<R>(&mut self, lexer: &mut Lexer<R>) -> Res
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

    fn real<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        let value = std::str::from_utf8(&self.buffer)?.parse::<f64>()?;
        self.opcodes.push(opcode::REAL)?;
        self.opcodes.extend(value.to_be_bytes())?;
        self.lex(lexer) // Skip value.
    }

    fn boolean<R>(&mut self, lexer: &mut Lexer<R>, value: bool) -> Res
    where
        R: std::io::Read,
    {
        self.opcodes
            .push(if value { opcode::TRUE } else { opcode::FALSE })?;
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
            Token::Semicolon => self.opcodes.push(opcode::VOID)?,
            _ => {
                self.expression(lexer)?;
                self.expect(Token::Semicolon)?;
            }
        }

        self.lex(lexer)?; // Skip ';'.
        self.opcodes.push(opcode::RET)
    }

    fn break_stat<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip 'break'.

        match self.token {
            Token::Semicolon => self.opcodes.push(opcode::VOID)?,
            _ => {
                self.expression(lexer)?;
                self.expect(Token::Semicolon)?;
            }
        }
        
        self.lex(lexer)?; // Skip ';'.

        let address = self.opcodes.len();
        self.opcodes.extend([0; 5])?;

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
            Token::Semicolon => self.opcodes.push(opcode::VOID)?,
            _ => {
                self.expression(lexer)?;
                self.expect(Token::Semicolon)?;
            }
        }

        self.lex(lexer)?; // Skip ';'.

        if let Some(cycle_start) = self.cycles_starts.last().cloned() {
            self.jump(cycle_start)
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
                    self.opcodes.push(opcode::DROP)?;
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

        self.opcodes.extend([opcode::CALL, arguments as u8])
    }

    fn store(&mut self, index: u32, is_local: bool) -> Res {
        if index <= u8::MAX as u32 {
            self.opcodes.extend([
                if is_local { opcode::ST1 } else { opcode::GS1 },
                index as u8,
            ])
        } else if index <= 0xFFFF {
            self.opcodes
                .push(if is_local { opcode::ST2 } else { opcode::GS2 })?;
            self.opcodes.extend((index as u16).to_be_bytes())
        } else {
            self.opcodes
                .push(if is_local { opcode::ST4 } else { opcode::GS4 })?;
            self.opcodes.extend(index.to_le_bytes())
        }
    }

    fn assign<R>(&mut self, lexer: &mut Lexer<R>, index: u32, is_local: bool) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '='.
        self.expression(lexer)?;
        self.store(index, is_local)
    }

    fn load(&mut self, index: u32, is_local: bool) -> Res {
        if index <= u8::MAX as u32 {
            self.opcodes.extend([
                if is_local { opcode::LD1 } else { opcode::GL1 },
                index as u8,
            ])
        } else if index <= 0xFFFF {
            self.opcodes
                .push(if is_local { opcode::LD2 } else { opcode::GL2 })?;
            self.opcodes.extend((index as u16).to_be_bytes())
        } else {
            self.opcodes
                .push(if is_local { opcode::LD4 } else { opcode::GL4 })?;
            self.opcodes.extend(index.to_le_bytes())
        }
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
                self.opcodes.push(opcode::SET)
            }
            _ => self.opcodes.push(opcode::GET),
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
        self.load(index, is_local)?;
        self.opcodes.push(opcode::INC)?;
        self.store(index, is_local)
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
        self.load(index, is_local)?;
        self.opcodes.push(opcode::DEC)?;
        self.store(index, is_local)
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
                _ => self.load(index, true),
            }
        } else if let Some(index) = self.find_global(&self.buffer) {
            self.lex(lexer)?; // Skip identifier.
            match self.token {
                Token::Equal => self.assign(lexer, index, false),
                Token::PlusPlus => self.postfix_identifier_increment(lexer, index, false),
                Token::MinusMinus => self.postfix_identifier_decrement(lexer, index, false),
                _ => self.load(index, false),
            }
        } else if let Some(address) = self.find_function(&self.buffer) {
            self.lex(lexer)?; // Skip identifier.
            self.opcodes.push(opcode::PTR)?;
            self.opcodes.extend(address.to_be_bytes())
        } else if let Some(index) = self.natives.get_index(&self.buffer) {
            self.lex(lexer)?; // Skip identifier.
            self.opcodes.push(opcode::NAT)?;
            self.opcodes.extend(index.to_be_bytes())
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
        self.store(local_id, self.is_parsing_function)?;
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

    fn jump(&mut self, address: u32) -> Res {
        if address <= 0xFFFF {
            self.opcodes.push(opcode::JP2)?;
            self.opcodes.extend((address as u16).to_be_bytes())?;
            self.opcodes.extend([0; 2])
        } else {
            self.opcodes.push(opcode::JP4)?;
            self.opcodes.extend(address.to_be_bytes())
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

        let end_false_address = self.opcodes.len();
        self.opcodes.extend([0; 5])?;
        self.address_stack.push(end_false_address);
        address_count += 1;

        loop {
            self.expression_without_logic(lexer)?;

            let end_false_address = self.opcodes.len();
            self.opcodes.extend([0; 5])?;
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

        self.opcodes.push(opcode::TRUE)?;

        let end_address = self.opcodes.len();
        self.opcodes.extend([0; 5])?;

        for _ in 0..address_count {
            let address = self.address_stack.pop().unwrap();
            self.set_jf(address, self.opcodes.len());
        }

        self.opcodes.push(opcode::FALSE)?;

        self.set_jp(end_address, self.opcodes.len());

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

        let end_true_address = self.opcodes.len();
        self.opcodes.extend([0; 5])?;
        self.address_stack.push(end_true_address);
        address_count += 1;

        loop {
            self.expression_without_logic(lexer)?;

            let end_true_address = self.opcodes.len();
            self.opcodes.extend([0; 5])?;
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

        self.opcodes.push(opcode::FALSE)?;

        let end_address = self.opcodes.len();
        self.opcodes.extend([0; 5])?;

        for _ in 0..address_count {
            let address = self.address_stack.pop().unwrap();
            self.set_jt(address, self.opcodes.len());
        }

        self.opcodes.push(opcode::TRUE)?;

        self.set_jp(end_address, self.opcodes.len());

        Ok(())
    }

    fn while_stat<R>(&mut self, lexer: &mut Lexer<R>) -> Res
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
        self.cycles_starts.push(while_start);
        self.cycles_end_addresses_sizes.push(0);

        self.expect(Token::LeftBrace)?;
        self.block(lexer)?;
        self.jump(while_start)?;

        self.cycles_starts.pop().unwrap();
        let ends_size = self.cycles_end_addresses_sizes.pop().unwrap();

        for _ in 0..ends_size {
            let address = self.cycles_end_addresses.pop().unwrap();
            self.set_jp(address, self.opcodes.len());
        }

        self.set_jf(end_jf_address, self.opcodes.len());
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
        self.store(local_id, true)?;
        self.opcodes.push(opcode::DROP)?;

        self.expect(Token::Comma)?;
        self.lex(lexer)?; // Skip ','.

        self.opcodes.push(opcode::VOID)?;
        let start = self.opcodes.len();

        // Condition.
        self.expression(lexer)?;
        let end_address = self.opcodes.len();
        self.opcodes.extend([0; 5])?;
        self.opcodes.push(opcode::DROP)?;

        let skip_address = self.opcodes.len();
        self.opcodes.extend([0; 5])?;
        let step = self.opcodes.len();

        self.expect(Token::Comma)?;
        self.lex(lexer)?; // Skip ','.

        self.expression(lexer)?;
        self.opcodes.push(opcode::DROP)?;

        let end_step_address = self.opcodes.len();
        self.opcodes.extend([0; 5])?;

        self.set_jp(skip_address, self.opcodes.len());

        // Block.
        self.expect(Token::LeftBrace)?;
        self.block(lexer)?;

        self.jump(step)?;
        self.set_jp(end_step_address, self.opcodes.len());

        self.jump(start)?;

        self.set_jf(end_address, self.opcodes.len());

        self.exit_block();

        Ok(())
    }

    fn list<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '['.
        self.opcodes.push(opcode::LIST)?;

        while self.token != Token::RightBracket {
            self.expression(lexer)?;
            self.opcodes.push(opcode::ADD)?;
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

        let end_jp_address = self.opcodes.len();
        self.opcodes.extend([0; 5])?;

        self.expect(Token::Identifier)?;
        self.add_function(self.opcodes.len())?;
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

        self.opcodes.push(args_count as u8)?;
        let stack_size_address = self.opcodes.len();
        self.opcodes.extend([0; 4])?;

        self.expect(Token::LeftBrace)?;
        self.block(lexer)?;

        self.opcodes.push(opcode::RET)?;

        self.is_parsing_function = false;

        (self.function.stack_size() - args_count)
            .to_be_bytes()
            .iter()
            .cloned()
            .enumerate()
            .for_each(|(i, b)| self.opcodes[stack_size_address + i as u32] = b);

        self.function.clear();

        self.set_jp(end_jp_address, self.opcodes.len());

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
                self.opcodes.push(opcode::NOT)
            }
            Token::Minus => {
                self.lex(lexer)?; // Skip '-'.
                self.secondary(lexer)?;
                self.opcodes.push(opcode::NEG)
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

            self.ranges.insert(
                self.opcodes.len(),
                Pos {
                    range,
                    source_id: self.source_id,
                },
            );

            self.opcodes.push(opcode)?;
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
        self.source_id = source_id;
        let mut lexer = Lexer::new(read)?;
        self.lex(&mut lexer)?;

        let stack_size_address = self.opcodes.len();
        self.opcodes.extend([0; 4])?;

        self.is_parsing_function = false;

        self.statements(&mut lexer, Token::End, true)?;

        self.opcodes.push(opcode::RET)?;

        self.global
            .stack_size()
            .to_be_bytes()
            .iter()
            .cloned()
            .enumerate()
            .for_each(|(i, b)| self.opcodes[stack_size_address + i as u32] = b);

        Ok(())
    }

    pub fn into_chunk(self) -> Chunk {
        Chunk {
            opcodes: self.opcodes.into(),
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
