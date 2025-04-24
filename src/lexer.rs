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
    in_set: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.as_bytes(),
            pos: 0,
            in_set: false,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let byte = self.peek();
        if byte.is_some() {
            self.pos += 1
        }
        byte
    }

    pub fn next_token(&mut self) -> Result<Option<Token>> {
        let Some(byte) = self.advance() else {
            return Ok(None);
        };

        match byte {
            b'(' => Ok(Some(Token::LParen)),
            b')' => Ok(Some(Token::RParen)),
            b'.' => Ok(Some(Token::Any)),
            b'[' => {
                self.in_set = true;
                Ok(Some(Token::LBracket))
            }
            b']' => {
                if self.in_set {
                    self.in_set = false;
                    Ok(Some(Token::RBracket))
                } else {
                    Ok(Some(Token::Literal(byte)))
                }
            }
            b'^' => Ok(Some(Token::Caret)),
            b'$' => Ok(Some(Token::Dollar)),
            b'*' => Ok(Some(Token::Star)),
            b'+' => Ok(Some(Token::Plus)),
            b'?' => Ok(Some(Token::Question)),
            b'-' => Ok(Some(Token::Minus)),
            b'%' => {
                if self.in_set {
                    if let Some(next_byte_peek) = self.peek() {
                        if is_class_byte(next_byte_peek) {
                            self.advance();
                            Ok(Some(Token::Class(next_byte_peek)))
                        } else {
                            Ok(Some(Token::Literal(b'%')))
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
                        c if is_escapable_magic_byte(c) => Ok(Some(Token::Literal(c))),
                        c if is_class_byte(c) => Ok(Some(Token::Class(c))),
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
                            "malformed pattern (invalid use of '%%' in pattern: %{:?})",
                            next_byte
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
