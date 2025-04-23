use super::{
    super::{Parser, Result, engine::find_first_match},
    calculate_start_index,
};
use cfg_if::cfg_if;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn svec(items: &[&str]) -> Vec<String> {
        items.iter().map(|&s| s.to_string()).collect()
    }

    #[test]
    fn test_basic_match() {
        assert_eq!(
            r#match("hello world", "hello", None),
            Ok(Some(svec(&["hello"])))
        );
        assert_eq!(
            r#match("hello world", "world", None),
            Ok(Some(svec(&["world"])))
        );
        assert_eq!(r#match("hello world", "bye", None), Ok(None));
    }

    #[test]
    fn test_match_with_captures() {
        assert_eq!(
            r#match("hello world", "(hello)", None),
            Ok(Some(svec(&["hello"])))
        );
        assert_eq!(
            r#match("hello world", "(hello) (world)", None),
            Ok(Some(svec(&["hello", "world"])))
        );
        assert_eq!(
            r#match("123-456-7890", "(%d+)%-(%d+)%-(%d+)", None),
            Ok(Some(svec(&["123", "456", "7890"])))
        );
    }

    #[test]
    fn test_match_with_init() {
        assert_eq!(
            r#match("hello world", "world", Some(6)),
            Ok(Some(svec(&["world"])))
        );
        assert_eq!(
            r#match("hello world", "hello", Some(1)),
            Ok(Some(svec(&["hello"])))
        );
        assert_eq!(r#match("hello world", "hello", Some(2)), Ok(None));
    }

    #[test]
    fn test_match_patterns() {
        assert_eq!(r#match("abc123", "%a+", None), Ok(Some(svec(&["abc"]))));
        assert_eq!(r#match("abc123", "%d+", None), Ok(Some(svec(&["123"]))));
        assert_eq!(
            r#match("abc123", "(%a+)(%d+)", None),
            Ok(Some(svec(&["abc", "123"])))
        );
    }

    #[test]
    fn test_match_with_empty_captures() {
        assert_eq!(
            r#match("hello", "(h)()ello", None),
            Ok(Some(svec(&["h", ""])))
        );
    }
}
