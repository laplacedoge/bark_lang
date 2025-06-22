use std::mem::take;

enum State {
    Start,
    Identifier,
    Zero,
    Dot,
    Integer,
    Hexadecimal,
    Octal,
    Binary,
    Fractional,
    Exponent,
    Equals,
    Minus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum IntegerRepresentation {
    Decimal(Vec<u8>),
    Hexadecimal(Vec<u8>),
    Octal(Vec<u8>),
    Binary(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum FloatRepresentation {
    Decimal {
        integer: Vec<u8>,
        fractional: Vec<u8>,
    },
    Scientific {
        integer: Vec<u8>,
        fractional: Vec<u8>,
        exponent: Vec<u8>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Plus,
    Minus,
    Asterisk,
    ForwardSlash,
    Dot,
    Comma,
    Colon,
    Semicolon,
    Assign,
    Equals,
    RightArrow,
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,

    False,
    True,
    And,
    Or,
    Not,
    Xor,
    Else,
    Function,
    If,
    Let,
    Return,
    Lambda,

    Identifier(Box<Vec<u8>>),
    Integer(Box<IntegerRepresentation>),
    Float(Box<FloatRepresentation>),

    EOF,
}

struct Lexer {
    state: State,
    integer: Vec<u8>,
    fractional: Vec<u8>,
    exponent: Vec<u8>,
    identifier: Vec<u8>,
    tokens: Vec<Token>,
}

enum Action {
    Continue,
    Again
}

enum InternalError {
    UnexpectedByte,
    InvalidNumberDigit,
    LeadingZeroWithoutBase,
    InvalidHexadecimalDigit,
    InvalidOctalDigit,
    InvalidBinaryDigit,
    MissingDigitsAfterBasePrefix,
}

#[derive(Debug)]
pub enum Error {
    UnexpectedByte(usize),
    InvalidNumberDigit(usize),
    LeadingZeroWithoutBase(usize),
    InvalidHexadecimalDigit(usize),
    InvalidOctalDigit(usize),
    InvalidBinaryDigit(usize),
    MissingDigitsAfterBasePrefix(usize),
    MissingDigitsAfterExponentMark(usize),
}

impl Lexer {
    fn new() -> Self {
        Self {
            state: State::Start,
            integer: vec![],
            fractional: vec![],
            exponent: vec![],
            identifier: vec![],
            tokens: vec![],
        }
    }

    fn run_fsm_start(self: &mut Self, byte: u8) -> Result<Action, InternalError> {
        let token = match byte {
            b' ' | b'\t' | b'\r' | b'\n' => {
                return Ok(Action::Continue);
            },
            b'A'..=b'Z' | b'a'..=b'z' | b'_' => {
                self.identifier.push(byte);
                self.state = State::Identifier;
                return Ok(Action::Continue);
            },
            b'0' => {
                self.state = State::Zero;
                return Ok(Action::Continue);
            },
            b'1'..=b'9' => {
                self.integer.push(byte - b'0');
                self.state = State::Integer;
                return Ok(Action::Continue);
            },
            b'+' => Token::Plus,
            b'-' => {
                self.state = State::Minus;
                return Ok(Action::Continue);
            },
            b'*' => Token::Asterisk,
            b'/' => Token::ForwardSlash,
            b'.' => {
                self.state = State::Dot;
                return Ok(Action::Continue);
            },
            b',' => Token::Comma,
            b':' => Token::Colon,
            b';' => Token::Semicolon,
            b'=' => {
                self.state = State::Equals;
                return Ok(Action::Continue);
            },
            b'(' => Token::LeftParenthesis,
            b')' => Token::RightParenthesis,
            b'[' => Token::LeftBracket,
            b']' => Token::RightBracket,
            b'{' => Token::LeftBrace,
            b'}' => Token::RightBrace,
            _ => {
                return Err(InternalError::UnexpectedByte);
            },
        };
        self.tokens.push(token);
        Ok(Action::Continue)
    }

    fn classify_identifier(self: &mut Self) {
        let token = match self.identifier.as_slice() {
            b"and"      => Token::And,
            b"else"     => Token::Else,
            b"false"    => Token::False,
            b"function" => Token::Function,
            b"if"       => Token::If,
            b"lambda"   => Token::Lambda,
            b"let"      => Token::Let,
            b"not"      => Token::Not,
            b"or"       => Token::Or,
            b"return"   => Token::Return,
            b"true"     => Token::True,
            b"xor"      => Token::Xor,
            _           => Token::Identifier(Box::new(take(&mut self.identifier))),
        };

        self.identifier.clear();
        self.tokens.push(token);
    }

    fn run_fsm_identifier(self: &mut Self, byte: u8) -> Result<Action, InternalError> {
        match byte {
            b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'_' => {
                self.identifier.push(byte);
                Ok(Action::Continue)
            },
            _ => {
                self.classify_identifier();
                self.state = State::Start;
                Ok(Action::Again)
            },
        }
    }

    fn run_fsm_zero(self: &mut Self, byte: u8) -> Result<Action, InternalError> {
        match byte {
            b'x' => {
                self.state = State::Hexadecimal;
                Ok(Action::Continue)
            },
            b'o' => {
                self.state = State::Octal;
                Ok(Action::Continue)
            },
            b'b' => {
                self.state = State::Binary;
                Ok(Action::Continue)
            },
            b'.' => {
                self.integer.push(0);
                self.state = State::Fractional;
                Ok(Action::Continue)
            },
            b'0'..=b'9' => {
                Err(InternalError::LeadingZeroWithoutBase)
            },
            b'A'..=b'Z' | b'a'..=b'z' => {
                Err(InternalError::InvalidNumberDigit)
            },
            _ => {
                let integer = IntegerRepresentation::Decimal(vec![0]);
                self.tokens.push(Token::Integer(Box::new(integer)));
                self.state = State::Start;
                Ok(Action::Again)
            },
        }
    }

    fn run_fsm_dot(self: &mut Self, byte: u8) -> Result<Action, InternalError> {
        match byte {
            b'0'..=b'9' => {
                self.fractional.push(byte - b'0');
                self.state = State::Fractional;
                Ok(Action::Continue)
            },
            _ => {
                self.tokens.push(Token::Dot);
                self.state = State::Start;
                Ok(Action::Again)
            },
        }
    }

    fn run_fsm_integer(self: &mut Self, byte: u8) -> Result<Action, InternalError> {
        match byte {
            b'0'..=b'9' => {
                self.integer.push(byte - b'0');
                Ok(Action::Continue)
            },
            b'.' => {
                self.state = State::Fractional;
                Ok(Action::Continue)
            },
            b'e' => {
                self.state = State::Exponent;
                Ok(Action::Continue)
            }
            _ => {
                let integer = IntegerRepresentation::Decimal(take(&mut self.integer));
                self.tokens.push(Token::Integer(Box::new(integer)));
                self.state = State::Start;
                Ok(Action::Again)
            },
        }
    }

    fn run_fsm_hexadecimal(self: &mut Self, byte: u8) -> Result<Action, InternalError> {
        match byte {
            b'0'..=b'9' => {
                self.integer.push(byte - b'0');
                Ok(Action::Continue)
            },
            b'A'..=b'F' => {
                self.integer.push(10 + (byte - b'A'));
                Ok(Action::Continue)
            },
            b'a'..=b'f' => {
                self.integer.push(10 + (byte - b'a'));
                Ok(Action::Continue)
            },
            b'G'..=b'Z' | b'g'..=b'z' => {
                Err(InternalError::InvalidHexadecimalDigit)
            },
            _ => {
                if self.integer.len() == 0 {
                    Err(InternalError::MissingDigitsAfterBasePrefix)
                } else {
                    let integer = IntegerRepresentation::Hexadecimal(take(&mut self.integer));
                    self.tokens.push(Token::Integer(Box::new(integer)));
                    self.state = State::Start;
                    Ok(Action::Again)
                }
            },
        }
    }

    fn run_fsm_octal(self: &mut Self, byte: u8) -> Result<Action, InternalError> {
        match byte {
            b'0'..=b'7' => {
                self.integer.push(byte - b'0');
                Ok(Action::Continue)
            },
            b'8'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' => {
                Err(InternalError::InvalidOctalDigit)
            },
            _ => {
                if self.integer.len() == 0 {
                    Err(InternalError::MissingDigitsAfterBasePrefix)
                } else {
                    let integer = IntegerRepresentation::Octal(take(&mut self.integer));
                    self.tokens.push(Token::Integer(Box::new(integer)));
                    self.state = State::Start;
                    Ok(Action::Again)
                }
            },
        }
    }

    fn run_fsm_binary(self: &mut Self, byte: u8) -> Result<Action, InternalError> {
        match byte {
            b'0'..=b'1' => {
                self.integer.push(byte - b'0');
                Ok(Action::Continue)
            },
            b'2'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' => {
                Err(InternalError::InvalidBinaryDigit)
            },
            _ => {
                if self.integer.len() == 0 {
                    Err(InternalError::MissingDigitsAfterBasePrefix)
                } else {
                    let integer = IntegerRepresentation::Binary(take(&mut self.integer));
                    self.tokens.push(Token::Integer(Box::new(integer)));
                    self.state = State::Start;
                    Ok(Action::Again)
                }
            },
        }
    }

    fn run_fsm_fractional(self: &mut Self, byte: u8) -> Result<Action, InternalError> {
        match byte {
            b'0'..=b'9' => {
                self.fractional.push(byte - b'0');
                Ok(Action::Continue)
            },
            b'e' => {
                self.state = State::Exponent;
                Ok(Action::Continue)
            }
            _ => {
                let float = FloatRepresentation::Decimal {
                    integer: take(&mut self.integer),
                    fractional: take(&mut self.fractional),
                };
                self.tokens.push(Token::Float(Box::new(float)));
                self.state = State::Start;
                Ok(Action::Again)
            },
        }
    }

    fn run_fsm_exponent(self: &mut Self, byte: u8) -> Result<Action, InternalError> {
        match byte {
            b'0'..=b'9' => {
                self.exponent.push(byte - b'0');
                Ok(Action::Continue)
            },
            _ => {
                let float = FloatRepresentation::Scientific {
                    integer: take(&mut self.integer),
                    fractional: take(&mut self.fractional),
                    exponent: take(&mut self.exponent),
                };
                self.tokens.push(Token::Float(Box::new(float)));
                self.state = State::Start;
                Ok(Action::Again)
            },
        }
    }

    fn run_fsm_equals(self: &mut Self, byte: u8) -> Result<Action, InternalError> {
        match byte {
            b'=' => {
                self.tokens.push(Token::Equals);
                Ok(Action::Continue)
            },
            _ => {
                self.tokens.push(Token::Assign);
                self.state = State::Start;
                Ok(Action::Again)
            },
        }
    }

    fn run_fsm_minus(self: &mut Self, byte: u8) -> Result<Action, InternalError> {
        match byte {
            b'>' => {
                self.tokens.push(Token::RightArrow);
                Ok(Action::Continue)
            },
            _ => {
                self.tokens.push(Token::Minus);
                self.state = State::Start;
                Ok(Action::Again)
            },
        }
    }

    fn run_fsm(self: &mut Self, byte: u8) -> Result<Action, InternalError> {
        match self.state {
            State::Start        => self.run_fsm_start(byte),
            State::Identifier   => self.run_fsm_identifier(byte),
            State::Zero         => self.run_fsm_zero(byte),
            State::Dot          => self.run_fsm_dot(byte),
            State::Integer      => self.run_fsm_integer(byte),
            State::Hexadecimal  => self.run_fsm_hexadecimal(byte),
            State::Octal        => self.run_fsm_octal(byte),
            State::Binary       => self.run_fsm_binary(byte),
            State::Fractional   => self.run_fsm_fractional(byte),
            State::Exponent     => self.run_fsm_exponent(byte),
            State::Equals       => self.run_fsm_equals(byte),
            State::Minus        => self.run_fsm_minus(byte),
        }
    }

    fn feed_byte(self: &mut Self, byte: u8) -> Result<(), InternalError> {
        loop {
            match self.run_fsm(byte) {
                Ok(action) => match action {
                    Action::Continue => return Ok(()),
                    Action::Again => continue,
                },
                Err(error) => return Err(error),
            }
        }
    }

    fn feed_script(self: &mut Self, script: &[u8]) -> Result<(), Error> {
        for i in 0..script.len() {
            let byte = script[i];
            match self.feed_byte(byte) {
                Ok(()) => continue,
                Err(error) => return match error {
                    InternalError::UnexpectedByte =>
                        Err(Error::UnexpectedByte(i)),
                    InternalError::InvalidNumberDigit =>
                        Err(Error::InvalidNumberDigit(i)),
                    InternalError::LeadingZeroWithoutBase =>
                        Err(Error::LeadingZeroWithoutBase(i)),
                    InternalError::InvalidHexadecimalDigit =>
                        Err(Error::InvalidHexadecimalDigit(i)),
                    InternalError::InvalidOctalDigit =>
                        Err(Error::InvalidOctalDigit(i)),
                    InternalError::InvalidBinaryDigit =>
                        Err(Error::InvalidBinaryDigit(i)),
                    InternalError::MissingDigitsAfterBasePrefix =>
                        Err(Error::MissingDigitsAfterBasePrefix(i)),
                },
            }
        }

        Ok(())
    }

    fn feed_eof(self: &mut Self, script: &[u8]) -> Result<(), Error> {
        let script_len = script.len();
        match self.state {
            State::Start => {
                Ok(())
            },
            State::Identifier => {
                self.classify_identifier();
                Ok(())
            },
            State::Zero => {
                let integer = IntegerRepresentation::Decimal(vec![0]);
                self.tokens.push(Token::Integer(Box::new(integer)));
                Ok(())
            },
            State::Dot => {
                self.tokens.push(Token::Dot);
                Ok(())
            },
            State::Integer => {
                let integer = IntegerRepresentation::Decimal(take(&mut self.integer));
                self.tokens.push(Token::Integer(Box::new(integer)));
                Ok(())
            },
            State::Hexadecimal => {
                if self.integer.len() == 0 {
                    Err(Error::MissingDigitsAfterBasePrefix(script_len))
                } else {
                    let integer = IntegerRepresentation::Hexadecimal(take(&mut self.integer));
                    self.tokens.push(Token::Integer(Box::new(integer)));
                    Ok(())
                }
            },
            State::Octal => {
                if self.integer.len() == 0 {
                    Err(Error::MissingDigitsAfterBasePrefix(script_len))
                } else {
                    let integer = IntegerRepresentation::Octal(take(&mut self.integer));
                    self.tokens.push(Token::Integer(Box::new(integer)));
                    Ok(())
                }
            },
            State::Binary => {
                if self.integer.len() == 0 {
                    Err(Error::MissingDigitsAfterBasePrefix(script_len))
                } else {
                    let integer = IntegerRepresentation::Binary(take(&mut self.integer));
                    self.tokens.push(Token::Integer(Box::new(integer)));
                    Ok(())
                }
            },
            State::Fractional => {
                let float = FloatRepresentation::Decimal {
                    integer: take(&mut self.integer),
                    fractional: take(&mut self.fractional),
                };
                self.tokens.push(Token::Float(Box::new(float)));
                Ok(())
            },
            State::Exponent => {
                if self.exponent.len() == 0 {
                    Err(Error::MissingDigitsAfterExponentMark(script_len))
                } else {
                    let float = FloatRepresentation::Scientific {
                        integer: take(&mut self.integer),
                        fractional: take(&mut self.fractional),
                        exponent: take(&mut self.exponent),
                    };
                    self.tokens.push(Token::Float(Box::new(float)));
                    Ok(())
                }
            },
            State::Equals => {
                self.tokens.push(Token::Assign);
                Ok(())
            },
            State::Minus => {
                self.tokens.push(Token::Minus);
                Ok(())
            },
        }
    }
}

pub fn tokenize(script: &[u8]) -> Result<Vec<Token>, Error> {
    let mut lexer = Lexer::new();
    lexer.feed_script(script)?;
    lexer.feed_eof(script)?;
    Ok(take(&mut lexer.tokens))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut tokens: Vec<Token>;

        tokens = tokenize(b"0 +0 -0 47 +2 -117").unwrap();
        assert_eq!(tokens, vec![
            Token::Integer(Box::new(IntegerRepresentation::Decimal(vec![0]))),
            Token::Plus,
            Token::Integer(Box::new(IntegerRepresentation::Decimal(vec![0]))),
            Token::Minus,
            Token::Integer(Box::new(IntegerRepresentation::Decimal(vec![0]))),
            Token::Integer(Box::new(IntegerRepresentation::Decimal(vec![4, 7]))),
            Token::Plus,
            Token::Integer(Box::new(IntegerRepresentation::Decimal(vec![2]))),
            Token::Minus,
            Token::Integer(Box::new(IntegerRepresentation::Decimal(vec![1, 1, 7]))),
        ]);

        tokens = tokenize(b"0.0 3.14 0. 3. .0 .14 3.14e10 0.e1 3.e10 .14e10").unwrap();
        assert_eq!(tokens, vec![
            Token::Float(Box::new(FloatRepresentation::Decimal {
                integer: vec![0], fractional: vec![0],
            })),
            Token::Float(Box::new(FloatRepresentation::Decimal {
                integer: vec![3], fractional: vec![1, 4],
            })),
            Token::Float(Box::new(FloatRepresentation::Decimal {
                integer: vec![0], fractional: vec![],
            })),
            Token::Float(Box::new(FloatRepresentation::Decimal {
                integer: vec![3], fractional: vec![],
            })),
            Token::Float(Box::new(FloatRepresentation::Decimal {
                integer: vec![], fractional: vec![0],
            })),
            Token::Float(Box::new(FloatRepresentation::Decimal {
                integer: vec![], fractional: vec![1, 4],
            })),
            Token::Float(Box::new(FloatRepresentation::Scientific {
                integer: vec![3], fractional: vec![1, 4], exponent: vec![1, 0],
            })),
            Token::Float(Box::new(FloatRepresentation::Scientific {
                integer: vec![0], fractional: vec![], exponent: vec![1],
            })),
            Token::Float(Box::new(FloatRepresentation::Scientific {
                integer: vec![3], fractional: vec![], exponent: vec![1, 0],
            })),
            Token::Float(Box::new(FloatRepresentation::Scientific {
                integer: vec![], fractional: vec![1, 4], exponent: vec![1, 0],
            })),
        ]);

        tokens = tokenize(b"0x64 0o77 0b10100101").unwrap();
        assert_eq!(tokens, vec![
            Token::Integer(Box::new(IntegerRepresentation::Hexadecimal(vec![6, 4]))),
            Token::Integer(Box::new(IntegerRepresentation::Octal(vec![7, 7]))),
            Token::Integer(Box::new(IntegerRepresentation::Binary(vec![1, 0, 1, 0, 0, 1, 0, 1]))),
        ]);

        tokens = tokenize(b"let x = 123;").unwrap();
        assert_eq!(tokens, vec![
            Token::Let,
            Token::Identifier(Box::new(b"x".to_vec())),
            Token::Assign,
            Token::Integer(Box::new(IntegerRepresentation::Decimal(vec![1, 2, 3]))),
            Token::Semicolon,
        ]);
    }
}