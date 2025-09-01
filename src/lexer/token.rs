#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Literal(u8),        // Normal character byte
    EscapedLiteral(u8), // Escaped character byte (by `%`)
    Any,                // .
    Class(u8),          // %a, %d etc. (just the identifying byte)
    LParen,             // (
    RParen,             // )
    LBracket,           // [
    RBracket,           // ]
    Caret,              // ^
    Dollar,             // $
    Star,               // *
    Plus,               // +
    Question,           // ?
    Minus,              // - (shortest match quantifier)
    Percent,            // % (used for escapes like %%, %b, %f)
    Balanced(u8, u8),   // %bxy (stores x and y)
    Frontier,           // %f
    CaptureRef(u8),     // %1, %2 ... %9 (only relevant in gsub replacement, but lexer can spot it)
                        // Note: %0 is handled differently (whole match)
}

impl Token {
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

#[derive(Clone, Copy, Debug, Eq)]
pub struct PosToken {
    pub pos: usize,
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
