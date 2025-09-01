use super::{Captures, calculate_start_index};
use crate::{
    Parser, Result,
    engine::{MatchRanges, find_first_match},
};

/// The indices and optional captures of a found string.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Match<'a> {
    /// The start index of the found string. This will be 1-indexed if the
    /// `1-based` feature is enabled.
    pub start: usize,
    /// The end index of the found string. This will be an inclusive index
    /// if the `1-based` feature is enabled.
    pub end: usize,
    /// The captured string slices. If a capture did not result in any value,
    /// it will be an empty slice.
    pub captures: Captures<'a>,
}

// TODO: This exists only to avoid having to spend a bunch of time changing the
// unit tests
#[doc(hidden)]
impl<'a> From<(usize, usize, Captures<'a>)> for Match<'a> {
    fn from((start, end, captures): (usize, usize, Captures<'a>)) -> Self {
        Self {
            start,
            end,
            captures,
        }
    }
}

/// Corresponds to Lua 5.3 [`string.find`].
/// Returns 1-based or 0-based (see features [`1-based`] and [`0-based`]) indices (start, end) and captured strings. The [`init`] argument can be either 0-based or 1-based.
pub fn find<'a>(
    text_bytes: &'a [u8],
    pattern: &[u8],
    init: Option<isize>,
    plain: bool,
) -> Result<Option<Match<'a>>> {
    let byte_len = text_bytes.len();

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

        if let Some(rel_byte_pos) = text_bytes[start_byte_index..]
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
        let mut parser = Parser::new(pattern)?;
        let ast = parser.parse()?;

        match find_first_match(&ast, text_bytes, start_byte_index) {
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
                    .map(|range| range.into_bytes(text_bytes))
                    .collect(),
            })),
            None => Ok(None),
        }
    }
}
