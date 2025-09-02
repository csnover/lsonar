use crate::{
    Result,
    ast::{AstRoot, parse_pattern},
    engine::{Capture, find_first_match},
};
pub use repl::Repl;
use repl::process_replacement_string;
use std::{borrow::Cow, ops::Range};

mod repl;

/// A piecewise text substitution engine.
///
/// Whereas the [`gsub`] function performs substitution in one shot, this type
/// allows for iterative replacement of strings by separating the matching and
/// replacing parts.
pub struct GSub {
    pattern: AstRoot,
    replacements: usize,
    found: usize,
    result: Vec<u8>,
    last_pos: usize,
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
        let ast = parse_pattern(pattern)?;

        Ok(Self {
            pattern: ast,
            replacements: n.unwrap_or(usize::MAX),
            found: 0,
            result: Vec::new(),
            last_pos: 0,
            current: 0..0,
        })
    }

    /// Returns the final string and the number of replacements, consuming the
    /// engine.
    #[must_use]
    pub fn finish(mut self, input: &[u8]) -> (Vec<u8>, usize) {
        self.result.extend(&input[self.last_pos..]);
        (self.result, self.found)
    }

    /// Advances to the next match in the given input.
    pub fn next<'a>(&mut self, input: &'a [u8]) -> Option<Vec<Cow<'a, [u8]>>> {
        if self.replacements > 0
            && let Some(ranges) = find_first_match(&self.pattern, input, self.last_pos)
        {
            self.found += 1;
            self.replacements -= 1;
            self.current = ranges.full_match;
            Some(self.captures(input, &ranges.captures))
        } else {
            None
        }
    }

    /// Replaces the current match with the given replacement text. If the given
    /// replacement is `None`, the original match is kept in the string.
    pub fn replace(&mut self, input: &[u8], replacement: Option<&[u8]>) {
        self.result
            .extend(&input[self.last_pos..self.current.start]);
        if let Some(replacement) = replacement {
            self.result.extend(replacement);
        } else {
            self.result.extend(&input[self.current.clone()]);
        }

        self.last_pos = self.current.end;

        if self.current.is_empty() && self.last_pos < input.len() {
            self.result.extend(&input[self.last_pos..=self.last_pos]);
            self.last_pos += 1;
            self.replacements = 1;
        }
    }

    fn captures<'a>(&self, input: &'a [u8], captures: &[Capture]) -> Vec<Cow<'a, [u8]>> {
        core::iter::once(Cow::Borrowed(&input[self.current.clone()]))
            .chain(captures.iter().map(|range| range.clone().into_bytes(input)))
            .collect::<Vec<_>>()
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
    repl: Repl<'a>,
    n: Option<usize>,
) -> Result<(Vec<u8>, usize)> {
    let mut generator = GSub::new(pattern, n)?;
    while let Some(captures) = generator.next(s) {
        let replacement = match repl {
            Repl::String(repl_str) => Some(process_replacement_string(repl_str, &captures[1..])),
            Repl::Function(f) => f(&captures),
            Repl::Table(f) => {
                let full_match = captures[0].clone();
                let key = captures.get(1).cloned().unwrap_or(full_match.clone());
                f(key)
            }
        };
        generator.replace(s, replacement.as_deref());
    }

    Ok(generator.finish(s))
}
