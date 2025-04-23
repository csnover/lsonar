use super::super::{Parser, Result, engine::find_first_match};
use cfg_if::cfg_if;

/// Corresponds to Lua 5.3 [`string.find`].
/// Returns 1-based or 0-based (see features [`1-based`] and [`0-based`]) indices (start, end) and captured strings. The [`init`] argument can be either 0-based or 1-based.
pub fn find(
    text: &str,
    pattern: &str,
    init: Option<isize>,
    plain: bool,
) -> Result<Option<(usize, usize, Vec<String>)>> {
    let text_bytes = text.as_bytes();
    let byte_len = text_bytes.len();

    let start_byte_index = match init {
        Some(i) if i > 0 => {
            let i = if cfg!(feature = "1-based") { i - 1 } else { i };
            let i = i as usize;
            if i >= byte_len { byte_len } else { i }
        }
        Some(i) if i < 0 => {
            let abs_i = (-i) as usize;
            if abs_i > byte_len {
                0
            } else {
                byte_len.saturating_sub(abs_i)
            }
        }
        _ => 0,
    };

    if plain {
        if start_byte_index >= byte_len && !pattern.is_empty() {
            return Ok(None);
        }

        if pattern.is_empty() {
            cfg_if! {
                if #[cfg(all(feature = "1-based", not(feature = "0-based")))] {
                    return Ok(Some((start_byte_index + 1, start_byte_index, vec![]))); // 1-based
                } else if #[cfg(all(feature = "0-based", not(feature = "1-based")))] {
                    return Ok(Some((start_byte_index, start_byte_index, vec![]))); // 0-based
                } else {
                    compile_error!("supports only 1-based and 0-based indices")
                }
            }
        }

        if let Some(rel_byte_pos) = text_bytes[start_byte_index..]
            .windows(pattern.len())
            .position(|window| window == pattern.as_bytes())
        {
            let start_pos = start_byte_index + rel_byte_pos;
            let end_pos = start_pos + pattern.len() - 1; // End position is inclusive in Lua

            cfg_if! {
                if #[cfg(all(feature = "1-based", not(feature = "0-based")))] {
                    Ok(Some((start_pos + 1, end_pos + 1, vec![]))) // 1-based
                } else if #[cfg(all(feature = "0-based", not(feature = "1-based")))] {
                    Ok(Some((start_pos, end_pos, vec![]))) // 0-based
                } else {
                    compile_error!("supports only 1-based and 0-based indices")
                }
            }
        } else {
            Ok(None)
        }
    } else {
        let mut parser = Parser::new(pattern)?;
        let ast = parser.parse()?;

        match find_first_match(&ast, text_bytes, start_byte_index)? {
            Some((match_byte_range, captures_byte_ranges)) => {
                let start_pos = match_byte_range.start;
                let end_pos = match_byte_range.end.saturating_sub(1);

                let captured_strings: Vec<String> = captures_byte_ranges
                    .into_iter()
                    .filter_map(|maybe_range| {
                        maybe_range
                            .map(|range| String::from_utf8_lossy(&text_bytes[range]).into_owned())
                    })
                    .collect();

                cfg_if! {
                    if #[cfg(all(feature = "1-based", not(feature = "0-based")))] {
                        Ok(Some((start_pos + 1, end_pos + 1, captured_strings))) // 1-based
                    } else if #[cfg(all(feature = "0-based", not(feature = "1-based")))] {
                        Ok(Some((start_pos, end_pos, captured_strings))) // 0-based
                    } else {
                        compile_error!("supports only 1-based and 0-based indices")
                    }
                }
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn svec(items: &[&str]) -> Vec<String> {
        items.iter().map(|&s| s.to_string()).collect()
    }

    #[test]
    fn test_negative_byte_classes() {
        assert_eq!(find("a b\tc", "%S", None, false), Ok(Some((1, 1, vec![]))));
        assert_eq!(find("a b\tc", "%S+", None, false), Ok(Some((1, 1, vec![]))));
        assert_eq!(find(" b\tc", "%S", None, false), Ok(Some((2, 2, vec![]))));
        assert_eq!(find("123abc", "%D", None, false), Ok(Some((4, 4, vec![]))));
        assert_eq!(find("123abc", "%D+", None, false), Ok(Some((4, 6, vec![]))));
        assert_eq!(find("abc_123", "%W", None, false), Ok(Some((4, 4, vec![]))));
        assert_eq!(find("-abc-", "%W", None, false), Ok(Some((1, 1, vec![]))));
        assert_eq!(find("abc123", "%A", None, false), Ok(Some((4, 4, vec![]))));
        assert_eq!(find("abc123", "%A+", None, false), Ok(Some((4, 6, vec![]))));
        assert_eq!(find("你a", "%A", None, false), Ok(Some((1, 1, vec![]))));
        assert_eq!(find("a你b", "%W", None, false), Ok(Some((2, 2, vec![]))));
    }

    #[test]
    fn test_balanced_patterns() {
        assert_eq!(
            find("a(b(c)d)e", "%b()", None, false),
            Ok(Some((2, 8, vec![])))
        );

        assert_eq!(
            find("a{b{c}d}e", "%b{}", None, false),
            Ok(Some((2, 8, vec![])))
        );

        assert_eq!(
            find("a<b<c>d>e", "%b<>", None, false),
            Ok(Some((2, 8, vec![])))
        );

        assert_eq!(
            find("a(b(c(d)e)f)g", "%b()", None, false),
            Ok(Some((2, 12, vec![])))
        );

        assert_eq!(
            find("a(b(c)d)e", "(%b())", None, false),
            Ok(Some((2, 8, svec(&["(b(c)d)"]))))
        );
    }

    #[test]
    fn test_find_invalid_pattern() {
        assert!(matches!(
            find("abc", "[", None, false),
            Err(Error::Parser(_))
        ));
        assert!(matches!(
            find("abc", "(", None, false),
            Err(Error::Parser(_))
        ));
        assert!(matches!(
            find("abc", "*", None, false),
            Err(Error::Parser(_))
        ));
        assert!(matches!(
            find("abc", "%", None, false),
            Err(Error::Lexer(_))
        ));
        assert!(matches!(
            find("abc", "%z", None, false),
            Err(Error::Parser(_)) | Err(Error::Lexer(_))
        ));
    }
}
