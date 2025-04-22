#![allow(clippy::type_complexity)]
#![allow(clippy::manual_is_ascii_check)]

pub mod ast;
pub mod charset;
pub mod engine;
pub mod lexer;
pub mod parser;
pub mod lua;

pub use self::{
    ast::{AstNode, Quantifier},
    charset::CharSet,
    lua::find,
    lexer::{Lexer, Token},
    parser::Parser,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Lexer(String),
    Parser(String),
    Matcher(String), // TODO: Maybe remove Matcher variant if engine returns Option/Result
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Lexer(s) | Error::Parser(s) | Error::Matcher(s) => write!(f, "{}", s),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub const LUA_MAXCAPTURES: usize = 32;

