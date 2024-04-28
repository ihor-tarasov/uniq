use crate::{lexer::Lexer, token::Token, Node, Operator};

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
                i32::MIN,
                i32::MAX
            )),
            token => Err(format!("Expected value, found {token}.")),
        }
    }

    fn binary_helper<N, O>(&mut self, next: &N, oper: &O) -> Result<Node, String>
    where
        N: Fn(&mut Self) -> Result<Node, String>,
        O: Fn(&Token) -> Option<Operator>,
    {
        let mut node = next(self)?;
        while let Some(operator) = oper(&self.token) {
            self.advance();
            let right = self.binary_helper(next, oper)?;
            node = Node::new_binary(node, right, operator);
        }
        Ok(node)
    }

    fn factor_operator(token: &Token) -> Option<Operator> {
        match token {
            Token::Asterisk => Some(Operator::Multiply),
            Token::Slash => Some(Operator::Divide),
            _ => None,
        }
    }

    fn factor(&mut self) -> Result<Node, String> {
        self.binary_helper(&Self::primary, &Self::factor_operator)
    }

    fn term_operator(token: &Token) -> Option<Operator> {
        match token {
            Token::Plus => Some(Operator::Addict),
            Token::Minus => Some(Operator::Subtract),
            _ => None,
        }
    }

    fn term(&mut self) -> Result<Node, String> {
        self.binary_helper(&Self::factor, &Self::term_operator)
    }

    fn binary(&mut self) -> Result<Node, String> {
        self.term()
    }

    pub fn parse(&mut self) -> Result<Option<Node>, String> {
        if self.token == Token::End {
            Ok(None)
        } else {
            let node = self.binary()?;
            match self.next() {
                Token::Unknown(c) => Err(format!(
                    "Expected end, found unknown character '{}'.",
                    c as char
                )),
                Token::ToBigInteger => Err(format!(
                    "Integer to big, supported range is from {} to {}",
                    i32::MIN,
                    i32::MAX
                )),
                Token::End => Ok(Some(node)),
                token => Err(format!("Expected end, found {token}")),
            }
        }
    }
}
