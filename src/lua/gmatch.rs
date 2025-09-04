use super::calculate_start_index;
use crate::{
    Result,
    ast::{AstRoot, parse_pattern},
    engine::{MatchRanges, find_first_match},
    lua::Capture,
};
use std::borrow::Cow;

/// Like Lua
/// [`string.gmatch`](https://www.lua.org/manual/5.3/manual.html#pdf-string.gmatch),
/// returns an iterator of the captures of `pattern` over the string `s`.
///
/// # Errors
///
/// If the pattern string could not be parsed, an [`Error`](crate::Error) is returned.
///
/// # Feature flags
///
/// Captured string positions are 1-indexed if the `1-based` feature is enabled.
pub fn gmatch<'a>(s: &'a [u8], pattern: &[u8], init: Option<isize>) -> Result<GMatchIterator<'a>> {
    let pattern_ast = parse_pattern(pattern)?;

    Ok(GMatchIterator {
        bytes: s,
        pattern_ast,
        current_pos: calculate_start_index(s.len(), init),
    })
}

pub struct GMatchIterator<'a> {
    pub(super) bytes: &'a [u8],
    pub(super) pattern_ast: AstRoot,
    pub(super) current_pos: usize,
}

impl<'a> Iterator for GMatchIterator<'a> {
    type Item = Vec<Capture<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pos > self.bytes.len() {
            return None;
        }

        find_first_match(&self.pattern_ast, self.bytes, self.current_pos).map(
            |MatchRanges {
                 full_match,
                 captures,
             }| {
                self.current_pos = full_match.end;
                if full_match.is_empty() {
                    self.current_pos += 1;
                }

                if captures.is_empty() {
                    vec![Cow::Borrowed(&self.bytes[full_match])]
                } else {
                    captures
                        .into_iter()
                        .map(|range| range.into_bytes(self.bytes))
                        .collect()
                }
            },
        )
    }
}
