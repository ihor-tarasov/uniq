use crate::raise;

use super::{Compiler, lexer::Lexer, token::Token, Res};

impl<'a> Compiler<'a> {
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

    pub(super) fn logic_and<R>(&mut self, lexer: &mut Lexer<R>, is_global: bool) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip 'and'.

        let mut address_count = 0;

        let end_false_address = self.chunk.empty_address()?;
        self.address_stack.push(end_false_address);
        address_count += 1;

        loop {
            self.expression_without_logic(lexer, is_global)?;

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

    pub(super) fn logic_or<R>(&mut self, lexer: &mut Lexer<R>, is_global: bool) -> Res
    where
        R: std::io::Read,
    {
        self.lex(lexer)?; // Skip 'and'.

        let mut address_count = 0;

        let end_true_address = self.chunk.empty_address()?;
        self.address_stack.push(end_true_address);
        address_count += 1;

        loop {
            self.expression_without_logic(lexer, is_global)?;

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
}
