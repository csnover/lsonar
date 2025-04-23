use super::{super::CharSet, Quantifier};

#[derive(Clone, PartialEq, Debug)]
pub enum AstNode {
    Literal(u8),
    Any,               // .
    Class(u8, bool),   // Class byte (e.g., b'a'), negated? (e.g. %A -> (b'a', true))
    Set(CharSet),      // Represents [...] or [^...]. CharSet handles negation internally.
    Balanced(u8, u8),  // %bxy
    Frontier(CharSet), // %f[...]

    // Anchors (zero-width assertions)
    AnchorStart, // ^
    AnchorEnd,   // $

    // Captures
    Capture {
        index: usize,        // 1-based index for Lua compatibility
        inner: Vec<AstNode>, // The nodes inside the capture
    },

    // Capture reference used in replacement string for gsub
    CaptureRef(usize), // %1, %2, ..., %9 (1-based index for Lua compatibility)

    // Quantified items
    Quantified {
        item: Box<AstNode>, // The node being quantified
        quantifier: Quantifier,
    },
}
