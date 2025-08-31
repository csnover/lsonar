use crate::{AstRoot, engine::find_first_match, lua::Captures};

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
            let result = Some(vec![&self.bytes[self.current_pos..self.current_pos]]);
            self.current_pos += 1;
            return result;
        }

        find_first_match(&self.pattern_ast, self.bytes, self.current_pos).and_then(
            |(match_range, captures)| {
                if match_range.start == match_range.end {
                    self.current_pos = match_range.end + 1;
                    if self.current_pos > self.bytes.len() {
                        return None;
                    }
                } else {
                    self.current_pos = match_range.end;
                }

                Some(if captures.is_empty() {
                    vec![&self.bytes[match_range]]
                } else {
                    captures
                        .into_iter()
                        .map(|range| &self.bytes[range])
                        .collect()
                })
            },
        )
    }
}
