//! A Lua-compatible string pattern matcher library.

#![warn(clippy::pedantic, rust_2018_idioms)]
#![allow(clippy::too_many_lines)]

pub mod ast;
pub mod charset;
mod engine;
pub mod lexer;
mod lua;
mod parser;

pub use self::lua::{Match, Repl, find, gmatch, gsub, r#match};

/// A pattern string parsing error.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    /// The pattern contains an invalid character set range or byte class.
    #[error("{err} at {pos}")]
    CharSet { pos: usize, err: charset::Error },

    /// The pattern ends in the middle of a character class.
    #[error("malformed pattern (ends with '%') at {pos}")]
    UnexpectedEnd { pos: usize },

    /// The pattern contains an unrecognised character class sequence.
    #[error("invalid escape sequence '%{}' at {pos}", lit.escape_ascii())]
    UnknownClass { pos: usize, lit: u8 },

    /// The pattern ends in the middle of a balanced pattern item.
    #[error("missing arguments to '%b' at {pos}")]
    MissingArgs { pos: usize },

    /// The number of capture groups in the pattern exceeds the supported number
    /// of captures.
    #[error("too many captures in pattern ({0} > {LUA_MAXCAPTURES})")]
    Captures(usize),

    /// A token of an unexpected type was encountered.
    #[error("unexpected '{}' at {pos}", lit.escape_ascii())]
    UnexpectedToken { pos: usize, lit: u8 },

    /// A token of an unexpected type was encountered.
    #[error("expected {expected:?}, got {actual:?} at {pos}")]
    ExpectedToken {
        pos: usize,
        expected: lexer::Token,
        actual: Option<lexer::Token>,
    },

    /// The pattern ends in the middle of an item.
    #[error("unexpected end of pattern at {pos}")]
    UnexpectedEndOfPattern { pos: usize },

    /// A bug occurred!
    #[error("internal error: percent token should not reach parser base")]
    InternalError { pos: usize },
}

/// The standard [`Result`](core::result::Result) type used by lsonar.
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// The maximum number of allowed capture groups in a pattern.
pub const LUA_MAXCAPTURES: usize = 32;
