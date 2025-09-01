use crate::{
    AstRoot,
    engine::{MatchRanges, find_first_match},
    lua::Captures,
};
use std::borrow::Cow;

pub struct GMatchIterator<'a> {
    pub(super) bytes: &'a [u8],
    pub(super) pattern_ast: AstRoot,
    pub(super) current_pos: usize,
    pub(super) is_empty_pattern: bool,
}

impl<'a> Iterator for GMatchIterator<'a> {
    type Item = Captures<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pos > self.bytes.len() {
            return None;
        }

        if self.is_empty_pattern {
            let result = Some(vec![Cow::Borrowed(
                &self.bytes[self.current_pos..self.current_pos],
            )]);
            self.current_pos += 1;
            return result;
        }

        find_first_match(&self.pattern_ast, self.bytes, self.current_pos).and_then(
            |MatchRanges {
                 full_match,
                 captures,
             }| {
                if full_match.is_empty() {
                    self.current_pos = full_match.end + 1;
                    if self.current_pos > self.bytes.len() {
                        return None;
                    }
                } else {
                    self.current_pos = full_match.end;
                }

                Some(if captures.is_empty() {
                    vec![Cow::Borrowed(&self.bytes[full_match])]
                } else {
                    captures
                        .into_iter()
                        .map(|range| range.into_bytes(self.bytes))
                        .collect()
                })
            },
        )
    }
}
