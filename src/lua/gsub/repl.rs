use crate::Error;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_replacement_string() {
        use ReplToken::*;

        let tokens = tokenize_replacement_string("hello %1 world %2");
        assert!(matches!(tokens[0], Literal(b'h')));
        assert!(matches!(tokens[6], CaptureRef(1)));
        assert!(matches!(tokens[14], CaptureRef(2)));

        let tokens = tokenize_replacement_string("%%");
        assert!(matches!(tokens[0], Literal(b'%')));

        let tokens = tokenize_replacement_string("%");
        assert!(matches!(tokens[0], Literal(b'%')));

        let tokens = tokenize_replacement_string("a%5b%c");
        assert!(matches!(tokens[0], Literal(b'a')));
        assert!(matches!(tokens[1], CaptureRef(5)));
        assert!(matches!(tokens[2], Literal(b'b')));
        assert!(matches!(tokens[3], Literal(b'%')));
        assert!(matches!(tokens[4], Literal(b'c')));
    }

    #[test]
    fn test_process_replacement_string() {
        assert_eq!(
            process_replacement_string("hello %1", &["world"]).unwrap(),
            "hello world"
        );

        assert_eq!(
            process_replacement_string("%2 is %1", &["name", "value"]).unwrap(),
            "value is name"
        );

        assert_eq!(process_replacement_string("%%", &[]).unwrap(), "%");

        assert_eq!(
            process_replacement_string("a%5b", &["1", "2", "3", "4", "5"]).unwrap(),
            "a5b"
        );

        assert_eq!(process_replacement_string("%9", &["capture"]).unwrap(), "");
    }
}
