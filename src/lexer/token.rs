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
