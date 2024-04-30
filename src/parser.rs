use crate::{lexer::Lexer, token::Token, Instruction, Node};

#[derive(PartialEq, PartialOrd, Clone, Copy)]
enum Precedence {
    Primary = 0,
    Term = 1,
    Factor = 2,
}

impl Precedence {
    fn next(self) -> Self {
        match self {
            Self::Primary => Self::Term,
            Self::Term => Self::Factor,
            Self::Factor => unreachable!(),
        }
    }
}

fn precedence_and_instruction_from_token(token: &Token) -> Option<(Precedence, Instruction)> {
    match token {
        Token::Plus => Some((Precedence::Term, Instruction::Addict)),
        Token::Minus => Some((Precedence::Term, Instruction::Subtract)),
        Token::Asterisk => Some((Precedence::Factor, Instruction::Multiply)),
        Token::Slash => Some((Precedence::Factor, Instruction::Divide)),
        Token::Percent => Some((Precedence::Factor, Instruction::Modulo)),
        _ => None,
    }
}

pub struct Parser<I> {
    lexer: Lexer<I>,
    token: Token,
}

impl<I> Parser<I>
where
    I: Iterator<Item = u8>,
{
    pub fn new(iter: I) -> Self {
        let mut lexer = Lexer::new(iter);
        Self {
            token: lexer.next(),
            lexer,
        }
    }

    fn advance(&mut self) {
        self.token = self.lexer.next();
    }

    fn next(&mut self) -> Token {
        std::mem::replace(&mut self.token, self.lexer.next())
    }

    fn primary(&mut self) -> Result<Node, String> {
        match self.next() {
            Token::Integer(value) => Ok(Node::new_integer(value)),
            Token::Unknown(c) => Err(format!(
                "Expected value, found unknown character '{}'.",
                c as char
            )),
            Token::End => Err(format!("Expected value, found end.")),
            Token::ToBigInteger => Err(format!(
                "Integer to big, supported range is from {} to {}",
                i64::MIN,
                i64::MAX
            )),
            token => Err(format!("Expected value, found {token}.")),
        }
    }

    fn binary(&mut self, expression_precedence: Precedence, mut left: Node) -> Result<Node, String> {
        while let Some((token_precedence, instruction)) = precedence_and_instruction_from_token(&self.token) {
            if token_precedence < expression_precedence {
                break;
            }
            self.advance();
            let mut right = self.primary()?;
            if let Some((next_precedence, _)) = precedence_and_instruction_from_token(&self.token) {
                if token_precedence < next_precedence {
                    right = self.binary(token_precedence.next(), right)?;
                }
            }
            left = Node::new_binary(left, right, instruction);
        }
        Ok(left)
    }

    fn expression(&mut self) -> Result<Node, String> {
        let left = self.primary()?;
        self.binary(Precedence::Primary, left)
    }

    pub fn parse(&mut self) -> Result<Option<Node>, String> {
        if self.token == Token::End {
            Ok(None)
        } else {
            let node = self.expression()?;
            match self.next() {
                Token::Unknown(c) => Err(format!(
                    "Expected end, found unknown character '{}'.",
                    c as char
                )),
                Token::ToBigInteger => Err(format!(
                    "Integer to big, supported range is from {} to {}",
                    i64::MIN,
                    i64::MAX
                )),
                Token::End => Ok(Some(node)),
                token => Err(format!("Expected end, found {token}")),
            }
        }
    }
}
