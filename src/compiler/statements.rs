use crate::{opcode, raise};

use super::{lexer::Lexer, token::Token, Compiler, Res};

impl<'a> Compiler<'a> {
    fn return_stat<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip 'return'.

        match self.token {
            Token::Semicolon => self.chunk.push(opcode::VOID)?,
            _ => {
                self.expression(lexer, false)?;
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
                self.expression(lexer, false)?;
                self.expect(Token::Semicolon)?;
            }
        }

        self.lex(lexer)?; // Skip ';'.

        let address = self.chunk.empty_address()?;

        if self.cycles.push_end(address) {
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
                self.expression(lexer, false)?;
                self.expect(Token::Semicolon)?;
            }
        }

        self.lex(lexer)?; // Skip ';'.

        if let Some(cycle_start) = self.cycles.start() {
            self.chunk.jump(cycle_start)
        } else {
            raise!("Unable to use 'continue' statement in this place.")
        }
    }

    pub(super) fn if_stat<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        let mut address_stack_size: u32 = 0;
        loop {
            self.lex(lexer)?; // Skip 'if'.
            self.expression(lexer, false)?; // Condition.

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

    pub(super) fn let_stat<R>(&mut self, lexer: &mut Lexer<R>, expect_semicolon: bool, is_global: bool) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip 'let'.
        self.expect(Token::Identifier)?;
        let id = if is_global {
            self.add_global()
        } else {
            self.add_local()
        };
        self.lex(lexer)?; // Skip vriable name.
        self.expect(Token::Equal)?;
        self.lex(lexer)?; // Skip '='.
        self.expression(lexer, is_global)?;
        self.chunk.store(id, is_global)?;
        if expect_semicolon {
            self.expect(Token::Semicolon)?;
            self.lex(lexer) // Skip ';'.
        } else {
            Ok(())
        }
    }

    pub(super) fn while_stat<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip 'while'.
        self.chunk.push(opcode::VOID)?;
        let while_start = self.chunk.len();
        // Condition.
        self.expression(lexer, false)?;
        let end_jf_address = self.chunk.empty_address()?;
        self.chunk.push(opcode::DROP)?;

        // Block.
        self.cycles.push_start(while_start);

        self.expect(Token::LeftBrace)?;
        self.block(lexer)?;
        self.chunk.jump(while_start)?;

        let ends_size = self.cycles.pop_start();

        for _ in 0..ends_size {
            let address = self.cycles.pop_end();
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

    pub(super) fn for_stat<R>(&mut self, lexer: &mut Lexer<R>) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip 'for'.

        self.enter_block();

        let local_id = self.add_local();
        self.lex(lexer)?; // Skip variable name.

        self.expect(Token::Equal)?;
        self.lex(lexer)?; // Skip '='.

        self.expression(lexer, false)?;
        self.chunk.store(local_id, false)?;
        self.chunk.push(opcode::DROP)?;

        self.expect(Token::Comma)?;
        self.lex(lexer)?; // Skip ','.

        self.chunk.push(opcode::VOID)?;
        let start = self.chunk.len();

        // Condition.
        self.expression(lexer, false)?;
        let end_address = self.chunk.empty_address()?;
        self.chunk.push(opcode::DROP)?;

        let skip_address = self.chunk.empty_address()?;
        let step = self.chunk.len();

        self.expect(Token::Comma)?;
        self.lex(lexer)?; // Skip ','.

        self.expression(lexer, false)?;
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

    pub(super) fn statements<R>(&mut self, lexer: &mut Lexer<R>, until: Token) -> Res
    where
        R: std::io::Read,
    {
        let mut first_time = true;
        loop {
            if first_time {
                first_time = false;
            } else {
                self.chunk.push(opcode::DROP)?;
            }

            match self.token {
                Token::Return => self.return_stat(lexer)?,
                Token::Break => self.break_stat(lexer)?,
                Token::Continue => self.continue_stat(lexer)?,
                Token::If => self.if_stat(lexer)?,
                Token::While => self.while_stat(lexer)?,
                Token::For => self.for_stat(lexer)?,
                Token::LeftBrace => self.block(lexer)?,
                Token::Let => self.let_stat(lexer, true, false)?,
                _ => {
                    self.expression(lexer, false)?;
                    self.expect(Token::Semicolon)?;
                    self.lex(lexer)?; // Skip ';'.
                }
            }

            if self.token == until {
                break Ok(());
            }
        }
    }
}
