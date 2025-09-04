use super::Capture;
use crate::{
    Error, Result,
    engine::{CaptureRange, find_first_match},
};
use std::{borrow::Cow, ops::Range};

/// A piecewise text substitution engine.
///
/// Whereas the [`gsub`] function performs substitution in one shot, this type
/// allows for iterative replacement of strings by separating the matching and
/// replacing parts.
pub struct GSub {
    pattern: Vec<u8>,
    replacements: usize,
    found: usize,
    result: Vec<u8>,
    last_pos: usize,
    last_replace: usize,
    current: Range<usize>,
}

impl GSub {
    /// Creates a new substitution engine.
    ///
    /// # Errors
    ///
    /// If the pattern string could not be parsed, an [`Error`](crate::Error) is
    /// returned.
    pub fn new(pattern: &[u8], n: Option<usize>) -> Result<Self> {
        Ok(Self {
            pattern: pattern.to_vec(),
            replacements: n.unwrap_or(usize::MAX),
            found: 0,
            result: Vec::new(),
            last_pos: 0,
            last_replace: usize::MAX,
            current: 0..0,
        })
    }

    /// Returns the final string and the number of replacements, consuming the
    /// engine.
    #[must_use]
    pub fn finish(mut self, input: &[u8]) -> (Vec<u8>, usize) {
        if let Some(input) = input.get(self.last_pos..) {
            self.result.extend(input);
        }
        (self.result, self.found)
    }

    /// Advances to the next match in the given input.
    ///
    /// # Errors
    ///
    /// If a syntax error is encountered in the pattern string, an [`Error`] is
    /// returned.
    pub fn next<'a>(&mut self, input: &'a [u8]) -> Result<Option<(Capture<'a>, Vec<Capture<'a>>)>> {
        Ok(
            if self.replacements > 0
                && let Some(ranges) = find_first_match(input, &self.pattern, self.last_pos)?
            {
                self.found += 1;
                self.replacements -= 1;
                self.current = ranges.full_match;
                Some(self.captures(input, &ranges.captures))
            } else {
                None
            },
        )
    }

    /// Replaces the current match with the given replacement text. If the given
    /// replacement is `None`, the original match is kept in the string.
    pub fn replace(&mut self, input: &[u8], replacement: Option<&[u8]>) {
        if self.current.end != self.last_replace {
            self.result
                .extend(&input[self.last_pos..self.current.start]);
            if let Some(replacement) = replacement {
                self.result.extend(replacement);
            } else {
                self.result.extend(&input[self.current.clone()]);
            }
        }

        self.last_replace = self.current.end;
        self.last_pos = self.current.end;

        if self.current.is_empty() {
            if let Some(input) = input.get(self.last_pos..=self.last_pos) {
                self.result.extend(input);
            }
            self.last_pos += 1;
            if self.last_pos == input.len() {
                self.replacements = 1;
            }
        }
    }

    fn captures<'a>(
        &self,
        input: &'a [u8],
        captures: &[CaptureRange],
    ) -> (Capture<'a>, Vec<Capture<'a>>) {
        (
            Cow::Borrowed(&input[self.current.clone()]),
            captures
                .iter()
                .map(|range| range.clone().into_bytes(input))
                .collect::<Vec<_>>(),
        )
    }
}

/// Like Lua
/// [`string.gsub`](https://www.lua.org/manual/5.3/manual.html#pdf-string.gsub),
/// returns a copy of `s` in which all (or the first `n`, if given) occurrences
/// of `pattern` are replaced by `repl`.
///
/// # Errors
///
/// If the pattern string could not be parsed, an [`Error`](crate::Error) is returned.
pub fn gsub<'a>(
    s: &'a [u8],
    pattern: &[u8],
    mut repl: Repl<'a>,
    n: Option<usize>,
) -> Result<(Vec<u8>, usize)> {
    let mut generator = GSub::new(pattern, n)?;
    while let Some((ref full_match, rest)) = generator.next(s)? {
        let replacement = match &mut repl {
            Repl::String(repl_str) => {
                Some(process_replacement_string(repl_str, full_match, &rest)?)
            }
            Repl::Function(f) => {
                let full_match = core::slice::from_ref(full_match);
                f(if rest.is_empty() { full_match } else { &rest })
            }
            Repl::Table(f) => {
                let key = rest.first().unwrap_or(full_match);
                f(key.clone())
            }
        };
        generator.replace(s, replacement.as_deref());
    }

    Ok(generator.finish(s))
}

type Key<'a> = Cow<'a, [u8]>;

/// The string replacement strategy to use with [`gsub`](crate::gsub).
pub enum Repl<'a> {
    /// The string value is used for replacement. The character `%` works as an
    /// escape character: any sequence in repl of the form `%d`, with `d`
    /// between 1 and 9, stands for the value of the `d`-th captured substring;
    /// the sequence `%0` stands for the whole match; the sequence `%%` stands
    /// for a single `%`.
    String(&'a [u8]),
    /// This function is called every time a match occurs, with all captured
    /// substrings passed as a slice, in order.
    Function(&'a mut dyn FnMut(&[Capture<'_>]) -> Option<Vec<u8>>),
    /// This function is queried for every match, using the first capture as the
    /// key.
    Table(&'a dyn Fn(Key<'_>) -> Option<Vec<u8>>),
}

enum ReplToken {
    Literal(u8),
    CaptureRef(u8),
}

fn process_replacement_string(
    repl: &[u8],
    full_match: &Capture<'_>,
    captures: &[Capture<'_>],
) -> Result<Vec<u8>> {
    let tokens = tokenize_replacement_string(repl)?;
    let mut result = Vec::with_capacity(tokens.len());

    for token in tokens {
        match token {
            ReplToken::Literal(b) => {
                result.push(b);
            }
            ReplToken::CaptureRef(idx) => {
                let idx = usize::from(idx);
                if idx == 0 || (idx == 1 && captures.is_empty()) {
                    result.extend(full_match.as_ref());
                } else if idx <= captures.len() {
                    result.extend(captures[idx - 1].as_ref());
                } else {
                    return Err(Error::InvalidCaptureIndex { pos: 0, index: idx });
                }
            }
        }
    }

    Ok(result)
}

fn tokenize_replacement_string(repl: &[u8]) -> Result<Vec<ReplToken>> {
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < repl.len() {
        if repl[i] == b'%' && i + 1 < repl.len() {
            tokens.push(match repl[i + 1] {
                next_byte if next_byte.is_ascii_digit() => ReplToken::CaptureRef(next_byte - b'0'),
                next_byte @ b'%' => ReplToken::Literal(next_byte),
                _ => return Err(Error::InvalidReplacement),
            });
            i += 2;
        } else {
            tokens.push(ReplToken::Literal(repl[i]));
            i += 1;
        }
    }

    Ok(tokens)
}
