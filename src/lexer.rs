use super::{Error, Result};

pub mod token;

pub use token::Token;

fn is_class_byte(c: u8) -> bool {
    matches!(
        c,
        b'a' | b'c'
            | b'd'
            | b'g'
            | b'l'
            | b'p'
            | b's'
            | b'u'
            | b'w'
            | b'x'
            | b'A'
            | b'C'
            | b'D'
            | b'G'
            | b'L'
            | b'P'
            | b'S'
            | b'U'
            | b'W'
            | b'X'
    )
}

fn is_escapable_magic_byte(c: u8) -> bool {
    matches!(
        c,
        b'(' | b')' | b'.' | b'%' | b'[' | b']' | b'*' | b'+' | b'-' | b'?' | b'^' | b'$'
    )
}

pub struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
    capture_depth: usize,
    set_depth: usize,
}

impl<'a> Lexer<'a> {
    #[must_use]
    pub fn new(input: &'a [u8]) -> Self {
        Lexer {
            input,
            pos: 0,
            capture_depth: 0,
            set_depth: 0,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let byte = self.peek();
        if byte.is_some() {
            self.pos += 1;
        }
        byte
    }

    pub fn next_token(&mut self) -> Result<Option<Token>> {
        let Some(byte) = self.advance() else {
            return Ok(None);
        };

        match byte {
            b'(' => {
                self.capture_depth += 1;
                Ok(Some(Token::LParen))
            }
            b')' => {
                if self.capture_depth > 0 {
                    self.capture_depth -= 1;
                    Ok(Some(Token::RParen))
                } else {
                    Ok(Some(Token::Literal(b')')))
                }
            }
            b'.' => Ok(Some(Token::Any)),
            b'[' => {
                self.set_depth += 1;
                Ok(Some(Token::LBracket))
            }
            b']' => {
                if self.set_depth > 0 {
                    self.set_depth -= 1;
                    Ok(Some(Token::RBracket))
                } else {
                    Ok(Some(Token::Literal(byte)))
                }
            }
            b'^' => Ok(Some(Token::Caret)),
            b'$' => Ok(Some(Token::Dollar)),
            b'*' => {
                if self.set_depth > 0 {
                    Ok(Some(Token::Literal(b'*')))
                } else {
                    Ok(Some(Token::Star))
                }
            }
            b'+' => {
                if self.set_depth > 0 {
                    Ok(Some(Token::Literal(b'+')))
                } else {
                    Ok(Some(Token::Plus))
                }
            }
            b'?' => {
                if self.set_depth > 0 {
                    Ok(Some(Token::Literal(b'?')))
                } else {
                    Ok(Some(Token::Question))
                }
            }
            b'-' => {
                if self.set_depth > 0 {
                    Ok(Some(Token::Literal(b'-')))
                } else {
                    Ok(Some(Token::Minus))
                }
            }
            b'%' => {
                if self.set_depth > 0 {
                    if let Some(next_byte) = self.peek() {
                        match next_byte {
                            byte if is_class_byte(next_byte) => Ok(Some(Token::Class(byte))),
                            byte if is_escapable_magic_byte(next_byte) => {
                                self.advance();
                                Ok(Some(Token::EscapedLiteral(byte)))
                            }
                            b'%' => {
                                self.advance();
                                Ok(Some(Token::EscapedLiteral(b'%')))
                            }
                            _ => Err(Error::Lexer(format!(
                                "malformed pattern (invalid escape sequence in set: %{next_byte})"
                            ))),
                        }
                    } else {
                        Err(Error::Lexer(
                            "malformed pattern (ends with '%' inside set)".to_string(),
                        ))
                    }
                } else {
                    let Some(next_byte) = self.advance() else {
                        return Err(Error::Lexer(
                            "malformed pattern (ends with '%')".to_string(),
                        ));
                    };
                    match next_byte {
                        c if is_escapable_magic_byte(c) => Ok(Some(Token::EscapedLiteral(c))),
                        c if is_class_byte(c) => Ok(Some(Token::Class(c))),
                        b'%' => {
                            self.advance();
                            Ok(Some(Token::EscapedLiteral(b'%')))
                        }
                        b'b' => {
                            let Some(d1) = self.advance() else {
                                return Err(Error::Lexer(
                                    "malformed pattern (%b needs two characters)".to_string(),
                                ));
                            };
                            let Some(d2) = self.advance() else {
                                return Err(Error::Lexer(
                                    "malformed pattern (%b needs two characters)".to_string(),
                                ));
                            };
                            Ok(Some(Token::Balanced(d1, d2)))
                        }
                        b'f' => Ok(Some(Token::Frontier)),
                        d @ b'1'..=b'9' => Ok(Some(Token::CaptureRef(d - b'0'))),
                        _ => Err(Error::Lexer(format!(
                            "malformed pattern (invalid escape sequence in set: %{next_byte})"
                        ))),
                    }
                }
            }

            _ => Ok(Some(Token::Literal(byte))),
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(Some(token)) => Some(Ok(token)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}
