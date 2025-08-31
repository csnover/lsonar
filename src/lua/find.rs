use super::{
    super::{engine::find_first_match, Parser, Result},
    calculate_start_index,
};

/// Corresponds to Lua 5.3 [`string.find`].
/// Returns 1-based or 0-based (see features [`1-based`] and [`0-based`]) indices (start, end) and captured strings. The [`init`] argument can be either 0-based or 1-based.
pub fn find(
    text_bytes: &[u8],
    pattern: &[u8],
    init: Option<isize>,
    plain: bool,
) -> Result<Option<(usize, usize, Vec<Vec<u8>>)>> {
    let byte_len = text_bytes.len();

    let start_byte_index = calculate_start_index(byte_len, init);

    if plain {
        if pattern.is_empty() {
            if cfg!(feature = "1-based") {
                return Ok(Some((
                    start_byte_index.saturating_add(1),
                    start_byte_index,
                    vec![],
                )));
            } else {
                return Ok(Some((start_byte_index, start_byte_index, vec![])));
            }
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

            let start_pos = if cfg!(feature = "1-based") {
                zero_based_start_pos.saturating_add(1)
            } else {
                zero_based_start_pos
            };

            let end_pos = zero_based_end_pos;

            Ok(Some((start_pos, end_pos, vec![])))
        } else {
            Ok(None)
        }
    } else {
        let mut parser = Parser::new(pattern)?;
        let ast = parser.parse()?;

        match find_first_match(&ast, text_bytes, start_byte_index)? {
            Some((match_byte_range, captures_byte_ranges)) => {
                let start_pos = if cfg!(feature = "1-based") {
                    match_byte_range.start.saturating_add(1)
                } else {
                    match_byte_range.start
                };
                let end_pos = match_byte_range.end;

                let captured_strings: Vec<Vec<u8>> = captures_byte_ranges
                    .into_iter()
                    .filter_map(|maybe_range| maybe_range.map(|range| text_bytes[range].to_owned()))
                    .collect();

                Ok(Some((start_pos, end_pos, captured_strings)))
            }
            None => Ok(None),
        }
    }
}
