use super::{
    super::{engine::find_first_match, Parser, Result},
    calculate_start_index,
};

/// Corresponds to Lua 5.3 `string.match`
pub fn r#match(text: &[u8], pattern: &[u8], init: Option<isize>) -> Result<Option<Vec<Vec<u8>>>> {
    let byte_len = text.len();

    let start_byte_index = calculate_start_index(byte_len, init);

    let mut parser = Parser::new(pattern)?;
    let ast = parser.parse()?;

    match find_first_match(&ast, text, start_byte_index)? {
        Some((match_byte_range, captures_byte_ranges)) => {
            let captures: Vec<_> = captures_byte_ranges
                .into_iter()
                .filter_map(|maybe_range| maybe_range.map(|range| text[range].to_owned()))
                .collect();

            if !captures.is_empty() {
                Ok(Some(captures))
            } else {
                let full_match = text[match_byte_range.start..match_byte_range.end].to_owned();
                Ok(Some(vec![full_match]))
            }
        }
        None => Ok(None),
    }
}
