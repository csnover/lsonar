use cfg_if::cfg_if;
use super::{calculate_start_index, super::{Result, Parser, engine::find_first_match}};

/// Corresponds to Lua 5.3 [`string.find`].
/// Returns 1-based or 0-based (see features [`1-based`] and [`0-based`]) indices (start, end) and captured strings. The [`init`] argument can be either 0-based or 1-based.
pub fn find(
    text: &str,
    pattern: &str,
    init: Option<isize>,
    plain: bool,
) -> Result<Option<(usize, usize, Vec<String>)>> {
    let text_bytes = text.as_bytes();
    let text_len = text.chars().count();
    let byte_len = text_bytes.len();

    let start_char_index = calculate_start_index(init, text_len);
    let start_byte_index = text.char_indices().nth(start_char_index).map_or(byte_len, |(idx, _)| idx);


    if plain {
        if start_byte_index >= byte_len && !pattern.is_empty() {
             return Ok(None);
        }

        if pattern.is_empty() {
            cfg_if! {
                if #[cfg(all(feature = "1-based", not(feature = "0-based")))] {
                    return Ok(Some((start_char_index + 1, start_char_index, vec![]))); // 1-based
                } else if #[cfg(all(feature = "0-based", not(feature = "1-based")))] {
                    return Ok(Some((start_char_index, start_char_index, vec![]))); // 0-based
                } else {
                    compile_error!("supports only 1-based and 0-based indices")
                }
            }
        }

        if let Some(rel_byte_pos) = text_bytes[start_byte_index..].windows(pattern.len()).position(|window| window == pattern.as_bytes()) {
            let abs_byte_pos = start_byte_index + rel_byte_pos;

            let start_char_pos = text[..abs_byte_pos].chars().count();
            let end_char_pos = start_char_pos + pattern.chars().count();
            
            cfg_if! {
                if #[cfg(all(feature = "1-based", not(feature = "0-based")))] {
                    Ok(Some((start_char_pos + 1, end_char_pos, vec![]))) // 1-based
                } else if #[cfg(all(feature = "0-based", not(feature = "1-based")))] {
                    Ok(Some((start_char_pos, end_char_pos, vec![]))) // 0-based
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
                let start_char_pos = text[..match_byte_range.start].chars().count();
                let end_char_pos = text[..match_byte_range.end].chars().count();
                let captured_strings: Vec<String> = captures_byte_ranges
                    .into_iter()
                    .filter_map(|maybe_range| {
                        maybe_range.map(|range| {
                            String::from_utf8_lossy(&text_bytes[range]).into_owned()
                        })
                    })
                    .collect();
                
                cfg_if! {
                    if #[cfg(all(feature = "1-based", not(feature = "0-based")))] {
                        Ok(Some((start_char_pos + 1, end_char_pos, captured_strings))) // 1-based
                    } else if #[cfg(all(feature = "0-based", not(feature = "1-based")))] {
                        Ok(Some((start_char_pos, end_char_pos, captured_strings))) // 0-based
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
    fn test_find_plain() {
        assert_eq!(find("hello world", "world", None, true), Ok(Some((7, 11, vec![]))));
        assert_eq!(find("hello world", "o", None, true), Ok(Some((5, 5, vec![]))));
        assert_eq!(find("hello world", "o", Some(6), true), Ok(Some((8, 8, vec![]))));
        assert_eq!(find("hello world", "l", Some(-5), true), Ok(Some((10, 10, vec![]))));
        assert_eq!(find("hello world", "notfound", None, true), Ok(None));
        assert_eq!(find("hello world", "o", Some(9), true), Ok(None));
        assert_eq!(find("aaaaa", "a", Some(3), true), Ok(Some((3, 3, vec![]))));
        assert_eq!(find("abc", "d", Some(1), true), Ok(None));
        assert_eq!(find("abc", "", Some(1), true), Ok(Some((1, 0, vec![]))));
        assert_eq!(find("abc", "", Some(3), true), Ok(Some((3, 2, vec![]))));
        assert_eq!(find("abc", "", Some(4), true), Ok(Some((4, 3, vec![]))));
        assert_eq!(find("", "", Some(1), true), Ok(Some((1, 0, vec![]))));
        assert_eq!(find("abc", "abc", Some(2), true), Ok(None));

        // UTF-8
        assert_eq!(find("你好世界", "好世", None, true), Ok(Some((2, 3, vec![]))));
        assert_eq!(find("你好世界", "界", Some(3), true), Ok(Some((4, 4, vec![]))));
        assert_eq!(find("你好世界", "界", Some(5), true), Ok(None));
        assert_eq!(find("你好世界", "你好", Some(-4), true), Ok(Some((1, 2, vec![]))));
        assert_eq!(find("你好世界", "", Some(3), true), Ok(Some((3, 2, vec![]))));
    }

    #[test]
    fn test_find_pattern_simple() {
        assert_eq!(find("hello world", "world", None, false), Ok(Some((7, 11, vec![]))));
        assert_eq!(find("hello world", "o.", None, false), Ok(Some((5, 6, vec![]))));
        assert_eq!(find("hello world", ".o", None, false), Ok(Some((4, 5, vec![]))));
        assert_eq!(find("hello world", "l+", None, false), Ok(Some((3, 4, vec![]))));
        assert_eq!(find("hello world", "l+", Some(4), false), Ok(Some((4, 4, vec![]))));
        assert_eq!(find("banana", "a.*a", None, false), Ok(Some((2, 6, vec![]))));
        assert_eq!(find("banana", "a.-a", None, false), Ok(Some((2, 4, vec![]))));
        assert_eq!(find("abc", "^a", None, false), Ok(Some((1, 1, vec![]))));
        assert_eq!(find("abc", "c$", None, false), Ok(Some((3, 3, vec![]))));
        assert_eq!(find("abc", "^b", None, false), Ok(None));
        assert_eq!(find("abc", "a$", None, false), Ok(None));
        assert_eq!(find("aaa", "a+", None, false), Ok(Some((1, 3, vec![]))));
        assert_eq!(find("", "^$", None, false), Ok(Some((1, 0, vec![]))));
        assert_eq!(find("abc", "x*", None, false), Ok(Some((1, 0, vec![]))));
        assert_eq!(find("abc", "x*", Some(2), false), Ok(Some((2, 1, vec![]))));
        assert_eq!(find("abc", "x*", Some(4), false), Ok(Some((4, 3, vec![]))));

        // UTF-8
        println!("first japanese test");
        assert_eq!(find("你好世界", "好.", None, false), Ok(Some((2, 3, vec![]))));
        println!("second japanese test");
        assert_eq!(find("你好世界", ".界", None, false), Ok(Some((3, 4, vec![]))));
        println!("thirst japanese test");
        assert_eq!(find("你好世界", "^你好", None, false), Ok(Some((1, 2, vec![]))));
        println!("fourth japanese test");
        assert_eq!(find("你好世界", "世界$", None, false), Ok(Some((3, 4, vec![]))));
        println!("fifth japanese test");
        assert_eq!(find("ééé", "é+", None, false), Ok(Some((1, 3, vec![]))));
    }

     #[test]
    fn test_find_pattern_captures() {
        assert_eq!(find("hello world", "(%w+)", None, false), Ok(Some((1, 5, svec(&["hello"])))) );
        assert_eq!(find("hello world", "(%w+) (%w+)", None, false), Ok(Some((1, 11, svec(&["hello", "world"])))) );
        assert_eq!(find("hello world", "(o.)", Some(6), false), Ok(Some((8, 9, svec(&["or"])))) );
        assert_eq!(find("aaa", "(a)", None, false), Ok(Some((1, 1, svec(&["a"])))) );
        assert_eq!(find("aaa", "(a*)", None, false), Ok(Some((1, 3, svec(&["aaa"])))) );
        assert_eq!(find("aaa", "a()a", None, false), Ok(Some((1, 2, svec(&[""])))) );
        assert_eq!(find("---", "(-*)", None, false), Ok(Some((1, 3, svec(&["---"])))) );
        assert_eq!(find("---", "(-?)", None, false), Ok(Some((1, 1, svec(&["-"])))) );

         assert_eq!(find("abc", "((.).)", None, false), Ok(Some((1, 3, svec(&["abc", "b"])))) );

         assert_eq!(find("a [b (c] d) e", "%b[]", None, false), Ok(Some((3, 11, vec![]))) );

        assert_eq!(find("abc", "a(b)c(d)?", None, false), Ok(Some((1, 3, svec(&["b"])))) );

        assert_eq!(find("你好世界", "(你好)(世界)", None, false), Ok(Some((1, 4, svec(&["你好", "世界"])))));
        assert_eq!(find("aa你好bb", "a+(%w+)b+", None, false), Ok(Some((1, 6, svec(&["你好"])))) );
    }

     #[test]
    fn test_find_invalid_pattern() {
        assert!(matches!(find("abc", "[", None, false), Err(Error::Parser(_))));
        assert!(matches!(find("abc", "(", None, false), Err(Error::Parser(_))));
        assert!(matches!(find("abc", "*", None, false), Err(Error::Parser(_))));
        assert!(matches!(find("abc", "%", None, false), Err(Error::Lexer(_))));
        assert!(matches!(find("abc", "%z", None, false), Err(Error::Parser(_)) | Err(Error::Lexer(_))));
    }

     #[test]
     fn test_find_index_handling() {
         assert_eq!(find("banana", "a", Some(1), false), Ok(Some((2, 2, vec![]))));
         assert_eq!(find("banana", "a", Some(2), false), Ok(Some((2, 2, vec![]))));
         assert_eq!(find("banana", "a", Some(3), false), Ok(Some((4, 4, vec![]))));
         assert_eq!(find("banana", "a", Some(6), false), Ok(Some((6, 6, vec![]))));
         assert_eq!(find("banana", "a", Some(7), false), Ok(None));
         assert_eq!(find("banana", "a", Some(0), false), Ok(Some((2, 2, vec![]))));

         assert_eq!(find("banana", "a", Some(-1), false), Ok(Some((6, 6, vec![]))));
         assert_eq!(find("banana", "a", Some(-2), false), Ok(Some((6, 6, vec![]))));
         assert_eq!(find("banana", "a", Some(-4), false), Ok(Some((4, 4, vec![]))));
         assert_eq!(find("banana", "a", Some(-6), false), Ok(Some((2, 2, vec![]))));
         assert_eq!(find("banana", "a", Some(-7), false), Ok(Some((2, 2, vec![]))));

         assert_eq!(find("", "a", Some(1), false), Ok(None));
         assert_eq!(find("", "a", Some(0), false), Ok(None));
         assert_eq!(find("", "a", Some(-1), false), Ok(None));

         assert_eq!(find("你好世界", "界", Some(1), false), Ok(Some((4, 4, vec![]))));
         assert_eq!(find("你好世界", "界", Some(4), false), Ok(Some((4, 4, vec![]))));
         assert_eq!(find("你好世界", "界", Some(5), false), Ok(None));
         assert_eq!(find("你好世界", "你", Some(-4), false), Ok(Some((1, 1, vec![]))));
         assert_eq!(find("你好世界", "好", Some(-3), false), Ok(Some((2, 2, vec![]))));
         assert_eq!(find("你好世界", "世", Some(-1), false), Ok(None));
     }
}