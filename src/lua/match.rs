use super::{
    super::{
        Parser, Result,
        engine::{MatchRanges, find_first_match},
    },
    Captures, calculate_start_index,
};
use std::borrow::Cow;

/// Corresponds to Lua 5.3 `string.match`
pub fn r#match<'a>(text: &'a [u8], pattern: &[u8], init: Option<isize>) -> Result<Captures<'a>> {
    let byte_len = text.len();

    let start_byte_index = calculate_start_index(byte_len, init);

    let mut parser = Parser::new(pattern)?;
    let ast = parser.parse()?;

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
