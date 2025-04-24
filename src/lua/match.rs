use super::{
    super::{Parser, Result, engine::find_first_match},
    calculate_start_index,
};

/// Corresponds to Lua 5.3 `string.match`
pub fn r#match(text: &str, pattern: &str, init: Option<isize>) -> Result<Option<Vec<String>>> {
    let text_bytes = text.as_bytes();
    let byte_len = text_bytes.len();

    let start_byte_index = calculate_start_index(byte_len, init);

    let mut parser = Parser::new(pattern)?;
    let ast = parser.parse()?;

    match find_first_match(&ast, text_bytes, start_byte_index)? {
        Some((match_byte_range, captures_byte_ranges)) => {
            let captures: Vec<_> = captures_byte_ranges
                .into_iter()
                .filter_map(|maybe_range| {
                    maybe_range
                        .map(|range| String::from_utf8_lossy(&text_bytes[range]).into_owned())
                })
                .collect();

            if !captures.is_empty() {
                Ok(Some(captures))
            } else {
                let full_match = String::from_utf8_lossy(
                    &text_bytes[match_byte_range.start..match_byte_range.end],
                )
                .into_owned();
                Ok(Some(vec![full_match]))
            }
        }
        None => Ok(None),
    }
}
