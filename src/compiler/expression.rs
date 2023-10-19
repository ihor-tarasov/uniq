use crate::{compiler::utils, opcode, raise};

use super::{lexer::Lexer, token::Token, Compiler, Pos, Res};

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

    fn subexpression<R>(&mut self, lexer: &mut Lexer<R>, is_global: bool) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '('.
        self.expression(lexer, is_global)?;
        self.expect(Token::RightParen)?;
        self.lex(lexer) // Skip ')'.
    }

    fn call<R>(&mut self, lexer: &mut Lexer<R>, is_global: bool) -> Res
    where
        R: std::io::Read,
    {
        let range = self.range.clone();
        self.lex(lexer)?; // Skip '('.

        let mut arguments = 0;
        loop {
            if arguments > u8::MAX as u32 {
                return raise!("Reached maximum function argumens number.");
            }

            if self.token == Token::RightParen {
                break;
            }

            self.expression(lexer, is_global)?;
            arguments += 1;

            if self.token == Token::Comma {
                self.lex(lexer)?; // Skip ','
            }
        }

        self.lex(lexer)?; // Skip ')'.

        self.chunk.push_pos(Pos {
            range,
            source_id: self.source_id,
        });
        self.chunk.call(arguments as u8)
    }

    fn assign<R>(&mut self, lexer: &mut Lexer<R>, index: u32, is_global: bool) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '='.
        self.expression(lexer, is_global)?;
        self.chunk.store(index, is_global)
    }

    fn index<R>(&mut self, lexer: &mut Lexer<R>, is_global: bool) -> Res
    where
        R: std::io::Read,
    {
        let range = self.range.clone();
        self.lex(lexer)?; // Skip '['.
        self.expression(lexer, is_global)?;
        self.expect(Token::RightBracket)?;
        self.lex(lexer)?; // Skip ']'.
        match self.token {
            Token::Equal => {
                self.lex(lexer)?; // Skip '='.
                self.expression(lexer, is_global)?;
                self.chunk.push_pos(Pos {
                    range,
                    source_id: self.source_id,
                });
                self.chunk.push(opcode::SET)
            }
            _ => {
                self.chunk.push_pos(Pos {
                    range,
                    source_id: self.source_id,
                });
                self.chunk.push(opcode::GET)
            }
        }
    }

    fn postfix_identifier_increment<R>(
        &mut self,
        lexer: &mut Lexer<R>,
        index: u32,
        is_global: bool,
    ) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '++'.
        self.chunk.load(index, is_global)?;
        self.chunk.load(index, is_global)?;
        self.chunk.push(opcode::INC)?;
        self.chunk.store(index, is_global)?;
        self.chunk.push(opcode::DROP)
    }

    fn postfix_identifier_decrement<R>(
        &mut self,
        lexer: &mut Lexer<R>,
        index: u32,
        is_global: bool,
    ) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '--'.
        self.chunk.load(index, is_global)?;
        self.chunk.load(index, is_global)?;
        self.chunk.push(opcode::DEC)?;
        self.chunk.store(index, is_global)?;
        self.chunk.push(opcode::DROP)
    }

    fn identifier_only<R>(&mut self, lexer: &mut Lexer<R>, is_global: bool) -> Res<(bool, u32)>
    where
        R: std::io::Read,
    {
        if !is_global {
            if let Some(index) = self.function.get(&self.buffer) {
                self.lex(lexer)?; // Skip identifier.
                return Ok((false, index));
            }
        }
        if let Some(index) = self.globals.get(&self.buffer) {
            self.lex(lexer)?; // Skip identifier.
            Ok((true, index))
        } else {
            raise!("Unknown identifier \"{}\".", utils::Slice(&self.buffer))
        }
    }

    fn prefix_identifier_increment<R>(&mut self, lexer: &mut Lexer<R>, is_global: bool) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '++'.
        let (is_global, index) = self.identifier_only(lexer, is_global)?;

        self.chunk.load(index, is_global)?;
        self.chunk.push(opcode::INC)?;
        self.chunk.store(index, is_global)
    }

    fn prefix_identifier_decrement<R>(&mut self, lexer: &mut Lexer<R>, is_global: bool) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '--'.
        self.expect(Token::Identifier)?;
        let (is_global, index) = self.identifier_only(lexer, is_global)?;

        self.chunk.load(index, is_global)?;
        self.chunk.push(opcode::DEC)?;
        self.chunk.store(index, is_global)
    }

    fn post_variable<R>(&mut self, lexer: &mut Lexer<R>, index: u32, is_global: bool) -> Res
    where
        R: std::io::Read,
    {
        match self.token {
            Token::Equal => self.assign(lexer, index, is_global),
            Token::PlusPlus => self.postfix_identifier_increment(lexer, index, is_global),
            Token::MinusMinus => self.postfix_identifier_decrement(lexer, index, is_global),
            _ => self.chunk.load(index, is_global),
        }
    }

    fn identifier<R>(&mut self, lexer: &mut Lexer<R>, is_global: bool) -> Res
    where
        R: std::io::Read,
    {
        if !is_global {
            if let Some(index) = self.function.get(&self.buffer) {
                self.lex(lexer)?; // Skip identifier.
                return self.post_variable(lexer, index, false);
            }
        }
        if let Some(index) = self.globals.get(&self.buffer) {
            self.lex(lexer)?; // Skip identifier.
            self.post_variable(lexer, index, true)
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

    fn list<R>(&mut self, lexer: &mut Lexer<R>, is_global: bool) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip '['.
        self.chunk.push(opcode::LIST)?;

        while self.token != Token::RightBracket {
            self.expression(lexer, is_global)?;
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

    fn primary<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        match self.token {
            Token::Integer => self.integer(lexer),
            Token::Real => self.real(lexer),
            Token::True => self.boolean(lexer, true),
            Token::False => self.boolean(lexer, false),
            Token::LeftParen => self.subexpression(lexer, false),
            Token::LeftBrace => self.block(lexer),
            Token::If => self.if_stat(lexer),
            Token::Let => self.let_stat(lexer, false, false),
            Token::Identifier => self.identifier(lexer, false),
            Token::While => self.while_stat(lexer),
            Token::LeftBracket => self.list(lexer, false),
            Token::For => self.for_stat(lexer),
            Token::PlusPlus => self.prefix_identifier_increment(lexer, false),
            Token::MinusMinus => self.prefix_identifier_decrement(lexer, false),
            Token::Unknown => raise!("Unknown token."),
            Token::End => raise!("Unexpected end."),
            _ => raise!("Unexpected token."),
        }
    }

    fn global_primary<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        match self.token {
            Token::Integer => self.integer(lexer),
            Token::Real => self.real(lexer),
            Token::True => self.boolean(lexer, true),
            Token::False => self.boolean(lexer, false),
            Token::LeftParen => self.subexpression(lexer, true),
            Token::Identifier => self.identifier(lexer, true),
            Token::LeftBracket => self.list(lexer, true),
            Token::PlusPlus => self.prefix_identifier_increment(lexer, true),
            Token::MinusMinus => self.prefix_identifier_decrement(lexer, true),
            Token::Unknown => raise!("Unknown token."),
            Token::End => raise!("Unexpected end."),
            _ => raise!("Unexpected token."),
        }
    }

    fn secondary<R>(&mut self, lexer: &mut Lexer<R>, is_global: bool) -> Res
    where
        R: std::io::Read,
    {
        if is_global {
            self.global_primary(lexer)?;
        } else {
            self.primary(lexer)?;
        }

        loop {
            match self.token {
                Token::LeftParen => self.call(lexer, is_global)?,
                Token::LeftBracket => self.index(lexer, is_global)?,
                _ => break Ok(()),
            }
        }
    }

    fn unary<R>(&mut self, lexer: &mut Lexer<R>, is_global: bool) -> Res
    where
        R: std::io::Read,
    {
        match self.token {
            Token::Exclamation => {
                self.lex(lexer)?; // Skip '!'.
                self.secondary(lexer, is_global)?;
                self.chunk.push(opcode::NOT)
            }
            Token::Minus => {
                self.lex(lexer)?; // Skip '-'.
                self.secondary(lexer, is_global)?;
                self.chunk.push(opcode::NEG)
            }
            _ => self.secondary(lexer, is_global),
        }
    }

    fn binary<R>(&mut self, lexer: &mut Lexer<R>, precedence: u8, is_global: bool) -> Res
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
            self.unary(lexer, is_global)?;

            let next = get_precedence(self.token);

            if current < next {
                self.binary(lexer, current + 1, is_global)?;
            }

            self.chunk.push_pos(Pos {
                range,
                source_id: self.source_id,
            });

            self.chunk.push(opcode)?;
        }
    }

    pub(super) fn expression_without_logic<R>(
        &mut self,
        lexer: &mut Lexer<R>,
        is_global: bool,
    ) -> Res
    where
        R: std::io::Read,
    {
        self.unary(lexer, is_global)?;
        self.binary(lexer, 1, is_global)
    }

    pub(super) fn expression<R>(&mut self, lexer: &mut Lexer<R>, is_global: bool) -> Res
    where
        R: std::io::Read,
    {
        self.expression_without_logic(lexer, is_global)?;

        match self.token {
            Token::And => self.logic_and(lexer, is_global),
            Token::Or => self.logic_or(lexer, is_global),
            _ => Ok(()),
        }
    }
}
