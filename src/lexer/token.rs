#[derive(Debug, Clone, PartialEq, Eq)]
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
