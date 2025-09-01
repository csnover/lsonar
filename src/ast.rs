//! Types for reading a pattern string as an abstract syntax tree.

pub use super::parser::parse_pattern;
use crate::charset::CharSet;

/// A syntax tree node.
#[derive(Clone, PartialEq, Debug)]
pub enum AstNode {
    /// A literal character.
    Literal(u8),
    /// Any character (`.`).
    Any,
    /// A character class.
    Class(
        /// The class byte (e.g. `b'a'` for `%a`). This value is always
        /// lowercase.
        u8,
        /// Whether the class is negated (e.g. `%A`, `%D`, etc.).
        bool,
    ),
    /// Represents `[...]` or `[^...]`. Negated forms are inverted by the parser
    /// so this set always represents the list of characters to match.
    Set(CharSet),
    /// A balanced pattern item in the form `%bxy`.
    Balanced(
        /// The opening byte.
        u8,
        /// The closing byte.
        u8,
    ),
    /// A frontier pattern in the form `%f[...]`.
    Frontier(CharSet),
    /// A start anchor (`^`).
    AnchorStart,
    /// A end anchor (`$`).
    AnchorEnd,
    /// A capture group.
    Capture {
        /// The index of the capture group. This is always a 1-based index.
        index: usize,
        /// The items inside the capture group.
        inner: Vec<AstNode>,
    },
    /// A capture reference used in replacement string for [`gsub`](crate::gsub).
    /// `%1`, `%2`, ..., `%9`. This is always a 1-based index.
    CaptureRef(usize),
    /// Quantified item group.
    Quantified {
        /// The items being quantified.
        item: Box<AstNode>,
        /// The item group quantifier.
        quantifier: Quantifier,
    },
}

/// A syntax tree root.
#[derive(Debug, Default)]
pub struct AstRoot {
    tree: Vec<AstNode>,
    capture_count: usize,
}

impl AstRoot {
    /// Creates a new tree from the given node list and capture count.
    #[must_use]
    pub(crate) fn new(tree: Vec<AstNode>, capture_count: usize) -> Self {
        Self {
            tree,
            capture_count,
        }
    }

    /// The total number of capture groups in the tree.
    #[must_use]
    pub fn capture_count(&self) -> usize {
        self.capture_count
    }
}

impl std::ops::Deref for AstRoot {
    type Target = [AstNode];

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}

// TODO: This exists only for unit tests
impl PartialEq<&[AstNode]> for AstRoot {
    fn eq(&self, other: &&[AstNode]) -> bool {
        self.tree == *other
    }
}

/// A specifier for pattern item repetitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quantifier {
    /// `*` (0 or more, greedy)
    Star,
    /// `+` (1 or more, greedy)
    Plus,
    /// `?` (0 or 1, greedy)
    Question,
    /// `-` (0 or more, non-greedy/shortest)
    Minus,
}
