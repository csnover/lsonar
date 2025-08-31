use super::{
    super::{Parser, Result, engine::find_first_match},
    Captures, calculate_start_index,
};

/// Corresponds to Lua 5.3 `string.match`
pub fn r#match<'a>(text: &'a [u8], pattern: &[u8], init: Option<isize>) -> Result<Captures<'a>> {
    let byte_len = text.len();

    let start_byte_index = calculate_start_index(byte_len, init);

    let mut parser = Parser::new(pattern)?;
    let ast = parser.parse()?;

    Ok(match find_first_match(&ast, text, start_byte_index) {
        Some((match_byte_range, captures_byte_ranges)) => {
            let captures = captures_byte_ranges;
            let has_captures = !captures.is_empty();

            if has_captures {
                captures.into_iter().map(|range| &text[range]).collect()
            } else {
                let full_match = &text[match_byte_range];
                vec![full_match]
            }
        }
        None => vec![],
    })
}
