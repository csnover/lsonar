pub mod token;

pub use super::{Error, Result};
pub use token::{PosToken, Token};

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
    inner: Inner<'a>,
    peek_token: Option<PosToken>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a [u8]) -> Result<Self> {
        let mut inner = Inner {
            input,
            pos: 0,
            capture_depth: 0,
            set_depth: 0,
        };

        let peek_token = inner.next()?;

        Ok(Self { inner, peek_token })
    }

    #[inline]
    #[must_use]
    pub fn tell(&self) -> usize {
        self.peek_token.map_or(self.inner.pos, |token| token.pos)
    }

    #[inline]
    #[must_use]
    pub fn peek(&self) -> Option<PosToken> {
        self.peek_token
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<PosToken>> {
        let next = self.peek_token;
        self.peek_token = self.inner.next()?;
        Ok(next)
    }

    #[inline]
    pub fn expect(&mut self, expected: Token) -> Result<()> {
        if self.consume(expected)? {
            Ok(())
        } else {
            Err(Error::ExpectedToken {
                pos: self.tell(),
                expected,
                actual: self.peek().map(|token| *token),
            })
        }
    }

    #[inline]
    pub fn consume(&mut self, other: Token) -> Result<bool> {
        let matches = self.next_is(other);
        if matches {
            self.next()?;
        }
        Ok(matches)
    }

    #[inline]
    pub fn until(&mut self, other: Token) -> Result<Option<PosToken>> {
        let matches = self.next_is(other);
        if matches { Ok(None) } else { self.next() }
    }

    #[inline]
    #[must_use]
    pub fn next_is(&self, other: Token) -> bool {
        self.peek_token.is_some_and(|token| *token == other)
    }
}

struct Inner<'a> {
    input: &'a [u8],
    pos: usize,
    capture_depth: usize,
    set_depth: usize,
}

impl Inner<'_> {
    fn next(&mut self) -> Result<Option<PosToken>> {
        let pos = self.pos;
        let Some(byte) = self.advance() else {
            return Ok(None);
        };

        let token = match byte {
            b'(' => {
                self.capture_depth += 1;
                Token::LParen
            }
            b')' => {
                if self.capture_depth > 0 {
                    self.capture_depth -= 1;
                    Token::RParen
                } else {
                    Token::Literal(b')')
                }
            }
            b'.' => Token::Any,
            b'[' => {
                self.set_depth += 1;
                Token::LBracket
            }
            b']' => {
                if self.set_depth > 0 {
                    self.set_depth -= 1;
                    Token::RBracket
                } else {
                    Token::Literal(byte)
                }
            }
            b'^' => Token::Caret,
            b'$' => Token::Dollar,
            b'*' => {
                if self.set_depth > 0 {
                    Token::Literal(b'*')
                } else {
                    Token::Star
                }
            }
            b'+' => {
                if self.set_depth > 0 {
                    Token::Literal(b'+')
                } else {
                    Token::Plus
                }
            }
            b'?' => {
                if self.set_depth > 0 {
                    Token::Literal(b'?')
                } else {
                    Token::Question
                }
            }
            b'-' => {
                if self.set_depth > 0 {
                    Token::Literal(b'-')
                } else {
                    Token::Minus
                }
            }
            b'%' => {
                if self.set_depth > 0 {
                    let Some(next_byte) = self.peek() else {
                        return Err(Error::UnexpectedEnd { pos: self.pos });
                    };
                    match next_byte {
                        byte if is_class_byte(byte) => Token::Class(byte),
                        byte if is_escapable_magic_byte(byte) => {
                            self.advance();
                            Token::EscapedLiteral(byte)
                        }
                        b'%' => {
                            self.advance();
                            Token::EscapedLiteral(b'%')
                        }
                        byte => {
                            return Err(Error::UnknownClass { pos, lit: byte });
                        }
                    }
                } else {
                    let Some(next_byte) = self.advance() else {
                        return Err(Error::UnexpectedEnd { pos: self.pos });
                    };
                    match next_byte {
                        c if is_escapable_magic_byte(c) => Token::EscapedLiteral(c),
                        c if is_class_byte(c) => Token::Class(c),
                        b'%' => {
                            self.advance();
                            Token::EscapedLiteral(b'%')
                        }
                        b'b' => {
                            let Some(d1) = self.advance() else {
                                return Err(Error::MissingArgs { pos: self.pos });
                            };
                            let Some(d2) = self.advance() else {
                                return Err(Error::MissingArgs { pos: self.pos });
                            };
                            Token::Balanced(d1, d2)
                        }
                        b'f' => Token::Frontier,
                        d @ b'1'..=b'9' => Token::CaptureRef(d - b'0'),
                        lit => {
                            return Err(Error::UnknownClass { pos, lit });
                        }
                    }
                }
            }

            _ => Token::Literal(byte),
        };

        Ok(Some(PosToken { pos, token }))
    }

    fn advance(&mut self) -> Option<u8> {
        let byte = self.peek();
        if byte.is_some() {
            self.pos += 1;
        }
        byte
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }
}
