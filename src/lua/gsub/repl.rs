use crate::Result;
use std::collections::HashMap;

pub enum Repl<'a> {
    String(&'a [u8]),
    Function(Box<dyn Fn(&[&[u8]]) -> Vec<u8> + 'a>),
    Table(&'a HashMap<&'a [u8], &'a [u8]>),
}

enum ReplToken {
    Literal(u8),
    CaptureRef(usize),
}

pub fn process_replacement_string(repl: &[u8], captures: &[&[u8]]) -> Result<Vec<u8>> {
    let tokens = tokenize_replacement_string(repl);
    let mut result = Vec::with_capacity(tokens.len());

    for token in tokens {
        match token {
            ReplToken::Literal(b) => {
                result.push(b);
            }
            ReplToken::CaptureRef(idx) => {
                if idx <= captures.len() {
                    result.extend(captures[idx - 1]);
                }
            }
        }
    }

    Ok(result)
}

fn tokenize_replacement_string(repl: &[u8]) -> Vec<ReplToken> {
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < repl.len() {
        if repl[i] == b'%' && i + 1 < repl.len() {
            let next_byte = repl[i + 1];
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
            tokens.push(ReplToken::Literal(repl[i]));
            i += 1;
        }
    }

    tokens
}
