use crate::Result;
use std::collections::HashMap;

pub enum Repl<'a> {
    String(&'a str),
    Function(Box<dyn Fn(&[&str]) -> String + 'a>),
    Table(&'a HashMap<String, String>),
}

enum ReplToken {
    Literal(u8),
    CaptureRef(usize),
}

pub fn process_replacement_string(repl: &str, captures: &[&str]) -> Result<String> {
    let tokens = tokenize_replacement_string(repl);
    let mut result = String::with_capacity(tokens.len());

    for token in tokens {
        match token {
            ReplToken::Literal(b) => {
                result.push(b as char);
            }
            ReplToken::CaptureRef(idx) => {
                if idx <= captures.len() {
                    result.push_str(captures[idx - 1]);
                }
            }
        }
    }

    Ok(result)
}

fn tokenize_replacement_string(repl: &str) -> Vec<ReplToken> {
    let mut tokens = Vec::new();
    let bytes = repl.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%' && i + 1 < bytes.len() {
            let next_byte = bytes[i + 1];
            if (b'1'..=b'9').contains(&next_byte) {
                let capture_idx = (next_byte - b'0') as usize;
                tokens.push(ReplToken::CaptureRef(capture_idx));
                i += 2;
            } else if next_byte == b'%' {
                tokens.push(ReplToken::Literal(b'%'));
                i += 2;
            } else {
                tokens.push(ReplToken::Literal(b'%'));
                i += 1;
            }
        } else {
            tokens.push(ReplToken::Literal(bytes[i]));
            i += 1;
        }
    }

    tokens
}
