//! A Lua-compatible string pattern matcher library.

#![warn(clippy::pedantic, rust_2018_idioms)]
#![allow(clippy::missing_errors_doc, clippy::too_many_lines)]

pub mod ast;
pub mod charset;
pub mod engine;
pub mod lexer;
pub mod lua;
pub mod parser;

pub use self::{
    ast::{AstNode, AstRoot, Quantifier},
    charset::CharSet,
    lexer::{Lexer, Token},
    lua::{Repl, find, gmatch, gsub, r#match},
    parser::Parser,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    /// An invalid character set range or byte class was used.
    #[error("{err} at {pos}")]
    CharSet { pos: usize, err: charset::Error },

    /// The pattern ended in the middle of a character class.
    #[error("malformed pattern (ends with '%') at {pos}")]
    UnexpectedEnd { pos: usize },

    /// The pattern contains an unrecognised character class sequence.
    #[error("invalid escape sequence '%{}' at {pos}", lit.escape_ascii())]
    UnknownClass { pos: usize, lit: u8 },

    /// The pattern ended in the middle of a balanced pattern item.
    #[error("missing arguments to '%b' at {pos}")]
    MissingArgs { pos: usize },

    /// The number of capture groups exceeds the supported number of captures.
    #[error("too many captures in pattern ({0} > {LUA_MAXCAPTURES})")]
    Captures(usize),

    /// A token of an unexpected type was encountered.
    #[error("unexpected '{}' at {pos}", lit.escape_ascii())]
    UnexpectedToken { pos: usize, lit: u8 },

    /// A token of an unexpected type was encountered.
    #[error("expected {expected:?}, got {actual:?} at {pos}")]
    ExpectedToken {
        pos: usize,
        expected: Token,
        actual: Option<Token>,
    },

    #[error("unexpected end of pattern at {pos}")]
    UnexpectedEndOfPattern { pos: usize },

    #[error("internal error: percent token should not reach parser base")]
    InternalError { pos: usize },
}

pub type Result<T> = std::result::Result<T, Error>;

pub const LUA_MAXCAPTURES: usize = 32;
