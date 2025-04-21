#![allow(clippy::type_complexity)]
#![allow(clippy::manual_is_ascii_check)]

pub mod ast;
pub mod charset;
pub mod engine;
pub mod lexer;
pub mod parser;

pub use self::{
    ast::{AstNode, Quantifier},
    charset::CharSet,
    engine::find_first_match,
    lexer::{Lexer, Token},
    parser::Parser,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Lexer(String),
    Parser(String),
    Matcher(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Lexer(s) | Error::Parser(s) | Error::Matcher(s) => write!(f, "{}", s),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub const LUA_MAXCAPTURES: usize = 32; // TODO: remove it??
