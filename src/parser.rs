use std::mem::take;
use crate::lexer::{Token, IntegerRepresentation, FloatRepresentation};
use crate::parser::Error::UnexpectedToken;

#[derive(Debug)]
pub struct UnaryOperation {
    operand: ASTNode,
}

#[derive(Debug)]
pub struct BinaryOperation {
    left_operand: ASTNode,
    right_operand: ASTNode,
}

#[derive(Debug)]
pub enum ASTNode {
    Identifier(Box<Vec<u8>>),
    IntegerLiteral(Box<IntegerRepresentation>),
    FloatLiteral(Box<FloatRepresentation>),
    UnaryAddition(Box<UnaryOperation>),
    UnarySubtraction(Box<UnaryOperation>),
    BinaryAddition(Box<BinaryOperation>),
    BinarySubtraction(Box<BinaryOperation>),
    BinaryMultiplication(Box<BinaryOperation>),
    BinaryDivision(Box<BinaryOperation>),
    LogicalAnd(Box<BinaryOperation>),
    LogicalOr(Box<BinaryOperation>),
    LogicalNot(Box<UnaryOperation>),
    LogicalXor(Box<BinaryOperation>),
    Assign(Box<BinaryOperation>),
}

#[derive(Debug)]
pub enum Error {
    UnexpectedToken,
}

pub struct Parser<'a> {
    tokens: &'a [Token],
    eof_token: Token,
    length: usize,
    offset: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            eof_token: Token::EOF,
            length: tokens.len(),
            offset: 0,
        }
    }

    fn peek(self: &Self) -> &Token {
        self.tokens.get(self.offset).unwrap_or(&self.eof_token)
    }

    fn advance(self: &mut Self) {
        if self.offset != self.length {
            self.offset += 1;
        }
    }

    fn consume(self: &mut Self) -> &Token {
        if let Some(token) = self.tokens.get(self.offset) {
            self.offset += 1;
            token
        } else {
            &self.eof_token
        }
    }

    fn expect(self: &Self, token: Token) {

    }

    fn parse(self: &mut Self) -> Result<ASTNode, Error> {
        match self.consume() {
            Token::Let => {
                match self.consume() {
                    Token::Identifier(identifier) => {
                        let identifier = ASTNode::Identifier(identifier.clone());
                        match self.consume() {
                            Token::Assign => {
                                let right_operand = self.parse_expression()?;
                                Ok(ASTNode::Assign(Box::new(BinaryOperation {
                                    left_operand: identifier, right_operand,
                                })))
                            },
                            _ => Err(UnexpectedToken),
                        }
                    },
                    _ => Err(UnexpectedToken),
                }
            },
            _ => Err(UnexpectedToken),
        }
    }

    fn parse_expression(self: &mut Self) -> Result<ASTNode, Error> {
        self.parse_term()
    }

    fn parse_term(self: &mut Self) -> Result<ASTNode, Error> {
        let mut operand = self.parse_factor()?;
        loop {
            match self.peek() {
                Token::Plus => {
                    self.consume();
                    let right_operand = self.parse_factor()?;
                    operand = ASTNode::BinaryAddition(Box::new(BinaryOperation {
                        left_operand: operand, right_operand,
                    }));
                },
                Token::Minus => {
                    self.consume();
                    let right_operand = self.parse_factor()?;
                    operand = ASTNode::BinarySubtraction(Box::new(BinaryOperation {
                        left_operand: operand, right_operand,
                    }));
                },
                _ => break,
            }
        }

        Ok(operand)
    }

    fn parse_factor(self: &mut Self) -> Result<ASTNode, Error> {
        let mut operand = self.parse_primary()?;
        loop {
            match self.peek() {
                Token::Asterisk => {
                    self.consume();
                    let right_operand = self.parse_primary()?;
                    operand = ASTNode::BinaryMultiplication(Box::new(BinaryOperation {
                        left_operand: operand, right_operand,
                    }));
                },
                Token::ForwardSlash => {
                    self.consume();
                    let right_operand = self.parse_primary()?;
                    operand = ASTNode::BinaryDivision(Box::new(BinaryOperation {
                        left_operand: operand, right_operand,
                    }));
                },
                _ => break,
            }
        }

        Ok(operand)
    }

    fn parse_primary(self: &mut Self) -> Result<ASTNode, Error> {
        match self.consume() {
            Token::Identifier(name) => {
                Ok(ASTNode::Identifier(name.clone()))
            },
            Token::Integer(integer) => {
                Ok(ASTNode::IntegerLiteral(integer.clone()))
            },
            Token::Float(float) => {
                Ok(ASTNode::FloatLiteral(float.clone()))
            },
            Token::LeftParenthesis => {
                let node = self.parse_expression()?;
                match self.consume() {
                    Token::RightParenthesis => {
                        Ok(node)
                    },
                    _ => Err(UnexpectedToken),
                }
            },
            _ => Err(UnexpectedToken),
        }
    }
}

pub fn parse(tokens: &[Token]) -> Result<ASTNode, Error> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}