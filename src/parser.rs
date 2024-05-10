use crate::{lexer::Lexer, source_error, token::Token, Instruction, Node, SourceResult};

#[derive(PartialEq, PartialOrd, Clone, Copy)]
enum Precedence {
    None = 0,
    Comparison = 1,
    Term = 2,
    Factor = 3,
}

impl Precedence {
    fn next(self) -> Self {
        match self {
            Self::None => Self::Comparison,
            Self::Comparison => Self::Term,
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
        Token::EqualsEquals => Some((Precedence::Comparison, Instruction::Equals)),
        Token::ExclamationEquals => Some((Precedence::Comparison, Instruction::NotEquals)),
        Token::Less => Some((Precedence::Comparison, Instruction::Less)),
        Token::Greater => Some((Precedence::Comparison, Instruction::Greater)),
        Token::LessEquals => Some((Precedence::Comparison, Instruction::LessEquals)),
        Token::GreaterEquals => Some((Precedence::Comparison, Instruction::GreaterEquals)),
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

    fn error<T>(&self, message: String) -> SourceResult<T> {
        source_error(message, self.lexer.location())
    }

    fn primary(&mut self) -> SourceResult<Node> {
        let result = match &self.token {
            Token::Integer(value) => Node::new_integer(*value),
            Token::Float(value) => Node::new_float(*value),
            Token::Unknown(c) => self.error(format!(
                "Expected value, found unknown character '{}'.",
                *c as char
            ))?,
            Token::End => self.error(format!("Expected value, found end."))?,
            Token::ToBigInteger => self.error(format!(
                "Integer to big, supported range is from {} to {}",
                i64::MIN,
                i64::MAX
            ))?,
            token => self.error(format!("Expected value, found {token}."))?,
        };
        self.advance();
        Ok(result)
    }

    fn binary(&mut self, expression_precedence: Precedence, mut left: Node) -> SourceResult<Node> {
        while let Some((token_precedence, instruction)) =
            precedence_and_instruction_from_token(&self.token)
        {
            if token_precedence < expression_precedence {
                break;
            }
            let location = self.lexer.location();
            self.advance();
            let mut right = self.primary()?;
            if let Some((next_precedence, _)) = precedence_and_instruction_from_token(&self.token) {
                if token_precedence < next_precedence {
                    right = self.binary(token_precedence.next(), right)?;
                }
            }
            left = Node::new_binary(left, right, instruction, location);
        }
        Ok(left)
    }

    fn expression(&mut self) -> SourceResult<Node> {
        let left = self.primary()?;
        self.binary(Precedence::None, left)
    }

    pub fn parse(&mut self) -> SourceResult<Option<Node>> {
        if self.token == Token::End {
            Ok(None)
        } else {
            let node = self.expression()?;
            match &self.token {
                Token::Unknown(c) => self.error(format!(
                    "Expected end, found unknown character '{}'.",
                    *c as char
                ))?,
                Token::ToBigInteger => self.error(format!(
                    "Integer to big, supported range is from {} to {}",
                    i64::MIN,
                    i64::MAX
                ))?,
                Token::End => Ok(Some(node)),
                token => self.error(format!("Expected end, found {token}"))?,
            }
        }
    }
}
