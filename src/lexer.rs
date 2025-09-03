//! Types for reading a pattern string as a token list.

use super::{Error, Result};

/// Converts a pattern string to tokens.
pub struct Lexer<'a> {
    inner: Inner<'a>,
    peek_token: Option<PosToken>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given `input`.
    ///
    /// # Errors
    ///
    /// If the first byte sequence in the input is not a valid token, an
    /// [`Error`] is returned.
    pub fn new(input: &'a [u8]) -> Result<Self> {
        let mut inner = Inner {
            input,
            pos: 0,
            capture_depth: 0,
        };

        let peek_token = inner.next()?;

        Ok(Self { inner, peek_token })
    }

    /// Returns the current position of the lexer in the input stream.
    #[inline]
    #[must_use]
    pub fn tell(&self) -> usize {
        self.peek_token.map_or(self.inner.pos, |token| token.pos)
    }

    /// Returns the next token from the input without consuming it.
    #[inline]
    #[must_use]
    pub fn peek(&self) -> Option<PosToken> {
        self.peek_token
    }

    /// Consumes and returns the next token from the input.
    ///
    /// # Errors
    ///
    /// If the next byte sequence in the input is not a valid token, an
    /// [`Error`] is returned.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<PosToken>> {
        let next = self.peek_token;
        self.peek_token = self.inner.next()?;
        Ok(next)
    }

    /// Consumes the next token from the input if it matches `expected`.
    ///
    /// # Errors
    ///
    /// If the next token does not match `expected`, or if the next byte
    /// sequence in the input is not a valid token, an [`Error`] is returned.
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

    /// Consumes and returns the next token from the input if it matches
    /// `expected`.
    ///
    /// # Errors
    ///
    /// If the token is consumed and the next byte sequence in the input is not
    /// a valid token, an [`Error`] is returned.
    #[inline]
    pub fn consume(&mut self, other: Token) -> Result<bool> {
        let matches = self.next_is(other);
        if matches {
            self.next()?;
        }
        Ok(matches)
    }

    /// Consumes and returns the next token from the input if it does *not*
    /// match `other`.
    ///
    /// # Errors
    ///
    /// If the token is consumed and the next byte sequence in the input is not
    /// a valid token, an [`Error`] is returned.
    #[inline]
    pub fn until(&mut self, other: Token) -> Result<Option<PosToken>> {
        let matches = self.next_is(other);
        if matches { Ok(None) } else { self.next() }
    }

    #[inline]
    #[must_use]
    fn next_is(&self, other: Token) -> bool {
        self.peek_token.is_some_and(|token| *token == other)
    }
}

struct Inner<'a> {
    input: &'a [u8],
    pos: usize,
    capture_depth: usize,
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
            b'[' => Token::LBracket,
            b']' => Token::RBracket,
            b'^' => Token::Caret,
            b'$' => Token::Dollar,
            b'*' => Token::Star,
            b'+' => Token::Plus,
            b'?' => Token::Question,
            b'-' => Token::Minus,
            b'%' => {
                let Some(next_byte) = self.advance() else {
                    return Err(Error::UnexpectedEnd { pos: self.pos });
                };
                match next_byte {
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
                    lit if is_class_byte(lit) => Token::Class(lit),
                    lit if is_escapable_magic_byte(lit) => Token::EscapedLiteral(lit),
                    lit => {
                        return Err(Error::UnknownClass { pos, lit });
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

/// A pattern string token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    /// A normal character byte.
    Literal(u8),
    /// A character byte escaped by `%`.
    EscapedLiteral(u8),
    /// `.`
    Any,
    /// A character class like `%a`, `%d` etc. (just the identifying byte)
    Class(u8),
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `^`
    Caret,
    /// `$`
    Dollar,
    /// `*`
    Star,
    /// `+`
    Plus,
    /// `?`
    Question,
    /// `-`. Shortest match quantifier.
    Minus,
    /// `%`. Used for escapes like `%%`, `%b`, `%f`.
    Percent,
    /// `%bxy`. `x` and `y` are stored.
    Balanced(u8, u8),
    /// `%f`
    Frontier,
    /// `%1`, `%2` ... `%9`. Only relevant in gsub replacement, but lexer can
    /// spot it. Note that `%0` is handled differently (whole match).
    CaptureRef(u8),
}

impl Token {
    /// Returns a byte representation of the token.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        match self {
            Token::Literal(b) | Token::EscapedLiteral(b) => b,
            Token::Any => b'.',
            Token::LParen => b'(',
            Token::RParen => b')',
            Token::LBracket => b'[',
            Token::RBracket => b']',
            Token::Caret => b'^',
            Token::Dollar => b'$',
            Token::Star => b'*',
            Token::Plus => b'+',
            Token::Question => b'?',
            Token::Minus => b'-',
            Token::Percent => b'%',
            Token::Class(c) => c,
            Token::Balanced(_, _) => b'b',
            Token::Frontier => b'f',
            Token::CaptureRef(d) => b'0' + d,
        }
    }
}

/// A [`Token`] with associated position information.
#[derive(Clone, Copy, Debug, Eq)]
pub struct PosToken {
    /// The start position of the token in the pattern string.
    pub pos: usize,
    /// The token.
    pub token: Token,
}

impl core::ops::Deref for PosToken {
    type Target = Token;

    fn deref(&self) -> &Self::Target {
        &self.token
    }
}

impl PartialEq for PosToken {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token
    }
}

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
            | b'z'
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
            | b'Z'
    )
}

fn is_escapable_magic_byte(c: u8) -> bool {
    !c.is_ascii_alphanumeric()
}
