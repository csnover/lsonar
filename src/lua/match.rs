use super::{Captures, calculate_start_index};
use crate::{
    Result,
    ast::parse_pattern,
    engine::{MatchRanges, find_first_match},
};
use std::borrow::Cow;

/// Like Lua
/// [`string.match`](https://www.lua.org/manual/5.3/manual.html#pdf-string.match),
/// looks for the first match of `pattern` in the string `s`.
///
/// # Errors
///
/// If the pattern string could not be parsed, an [`Error`](crate::Error) is returned.
///
/// # Feature flags
///
/// The input `init` index is 1-indexed if the `1-based` feature is enabled.
pub fn r#match<'a>(text: &'a [u8], pattern: &[u8], init: Option<isize>) -> Result<Captures<'a>> {
    let byte_len = text.len();

    let start_byte_index = calculate_start_index(byte_len, init);

    let ast = parse_pattern(pattern)?;

    Ok(match find_first_match(&ast, text, start_byte_index) {
        Some(MatchRanges {
            full_match,
            captures,
        }) => {
            let has_captures = !captures.is_empty();

            if has_captures {
                captures
                    .into_iter()
                    .map(|range| range.into_bytes(text))
                    .collect()
            } else {
                vec![Cow::Borrowed(&text[full_match])]
            }
        }
        None => vec![],
    })
}
