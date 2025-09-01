use super::{Captures, calculate_start_index};
use crate::{
    Result,
    ast::parse_pattern,
    engine::{MatchRanges, find_first_match},
};

/// The result of a [`find`] call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Match<'a> {
    /// The start index of the found string.
    ///
    /// # Feature flags
    ///
    /// This will be 1-indexed if the `1-based` feature is enabled.
    pub start: usize,
    /// The end index of the found string.
    ///
    /// # Feature flags
    ///
    /// This will be an inclusive index if the `1-based` feature is enabled.
    pub end: usize,
    /// The captured string slices. If a capture did not result in any value,
    /// it will be an empty slice.
    pub captures: Captures<'a>,
}

// TODO: This exists only to avoid having to spend a bunch of time changing the
// unit tests
impl<'a> From<(usize, usize, Captures<'a>)> for Match<'a> {
    fn from((start, end, captures): (usize, usize, Captures<'a>)) -> Self {
        Self {
            start,
            end,
            captures,
        }
    }
}

/// Like Lua
/// [`string.find`](https://www.lua.org/manual/5.3/manual.html#pdf-string.find),
/// looks for the first match of `pattern` in the string `s`.
///
/// # Errors
///
/// If the pattern string could not be parsed, an [`Error`](crate::Error) is returned.
///
/// # Feature flags
///
/// The input `init` and output `start` and `end` indices are 1-indexed if the
/// `1-based` feature is enabled.
pub fn find<'a>(
    s: &'a [u8],
    pattern: &[u8],
    init: Option<isize>,
    plain: bool,
) -> Result<Option<Match<'a>>> {
    let byte_len = s.len();

    let start_byte_index = calculate_start_index(byte_len, init);

    if plain {
        if pattern.is_empty() {
            return Ok(Some(Match {
                start: if cfg!(feature = "1-based") {
                    start_byte_index.saturating_add(1)
                } else {
                    start_byte_index
                },
                end: start_byte_index,
                captures: vec![],
            }));
        }

        if start_byte_index >= byte_len {
            return Ok(None);
        }

        if let Some(rel_byte_pos) = s[start_byte_index..]
            .windows(pattern.len())
            .position(|window| window == pattern)
        {
            let zero_based_start_pos = start_byte_index + rel_byte_pos;
            let zero_based_end_pos = zero_based_start_pos + pattern.len();

            Ok(Some(Match {
                start: if cfg!(feature = "1-based") {
                    zero_based_start_pos.saturating_add(1)
                } else {
                    zero_based_start_pos
                },
                end: zero_based_end_pos,
                captures: vec![],
            }))
        } else {
            Ok(None)
        }
    } else {
        let ast = parse_pattern(pattern)?;

        match find_first_match(&ast, s, start_byte_index) {
            Some(MatchRanges {
                full_match,
                captures,
            }) => Ok(Some(Match {
                start: if cfg!(feature = "1-based") {
                    full_match.start.saturating_add(1)
                } else {
                    full_match.start
                },
                end: full_match.end,
                captures: captures
                    .into_iter()
                    .map(|range| range.into_bytes(s))
                    .collect(),
            })),
            None => Ok(None),
        }
    }
}
