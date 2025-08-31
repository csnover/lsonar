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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Lexer(String),
    Parser(String),
    Matcher(String), // TODO: Maybe remove Matcher variant if engine returns Option/Result
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Lexer(s) | Error::Parser(s) | Error::Matcher(s) => write!(f, "{s}"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub const LUA_MAXCAPTURES: usize = 32;
