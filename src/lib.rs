//! A Lua-compatible string pattern matcher library.

#![warn(clippy::pedantic, rust_2018_idioms)]
#![allow(clippy::too_many_lines)]

mod engine;
mod lua;

pub use self::lua::{Capture, GSub, Match, Repl, find, gmatch, gsub, r#match};

/// A pattern string parsing error.
#[derive(Debug, Eq, thiserror::Error, PartialEq)]
pub enum Error {
    #[error("pattern too complex at {pos}")]
    TooComplex { pos: usize },
    #[error("too many captures at {pos}")]
    TooManyCaptures { pos: usize },
    #[error("invalid pattern capture at {pos}")]
    InvalidPatternCapture { pos: usize },
    #[error("missing '[' after '%f' in pattern at {pos}")]
    IncompleteFrontier { pos: usize },
    #[error("malformed pattern (missing arguments to '%b') at {pos}")]
    MissingBalanceArgs { pos: usize },
    #[error("invalid capture index %{index} at {pos}")]
    InvalidCaptureIndex { pos: usize, index: usize },
    #[error("malformed pattern (ends with '%') at {pos}")]
    EndsWithPercent { pos: usize },
    #[error("malformed pattern (missing ']') at {pos}")]
    EndsWithoutBracket { pos: usize },
    #[error("unfinished capture at {pos}")]
    UnfinishedCapture { pos: usize },
    #[error("invalid use of '%' in replacement string")]
    InvalidReplacement,
}

/// The standard [`Result`](core::result::Result) type used by lsonar.
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// The maximum number of allowed capture groups in a pattern.
pub const LUA_MAXCAPTURES: usize = 32;
