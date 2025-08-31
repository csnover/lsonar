type Key<'a> = &'a [u8];
type Captures<'a> = &'a [&'a [u8]];

/// String replacement strategy.
#[derive(Clone, Copy)]
pub enum Repl<'a> {
    /// The string value is used for replacement. The character `%` works as an
    /// escape character: any sequence in repl of the form `%d`, with `d`
    /// between 1 and 9, stands for the value of the `d`-th captured substring;
    /// the sequence `%0` stands for the whole match; the sequence `%%` stands
    /// for a single `%`.
    String(&'a [u8]),
    /// This function is called every time a match occurs, with all captured
    /// substrings passed as a slice, in order.
    Function(&'a dyn Fn(Captures<'_>) -> Vec<u8>),
    /// This function is queried for every match, using the first capture as the
    /// key.
    Table(&'a dyn Fn(Key<'_>) -> Option<Vec<u8>>),
}

enum ReplToken {
    Literal(u8),
    CaptureRef(usize),
}

pub fn process_replacement_string(repl: &[u8], captures: &[&[u8]]) -> Vec<u8> {
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

    result
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
