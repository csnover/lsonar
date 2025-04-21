use super::{super::CharSet, Quantifier};

#[derive(Clone, PartialEq, Debug)]
pub enum AstNode {
    // Basic matching units
    Literal(u8),
    Any,               // .
    Class(char, bool), // Class char (e.g., 'a'), negated? (e.g. %A -> ('a', true))
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

    // Quantified items
    Quantified {
        item: Box<AstNode>, // The node being quantified
        quantifier: Quantifier,
    },
}