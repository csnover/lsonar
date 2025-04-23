use super::super::{Parser, Result, engine::find_first_match};
use repl::process_replacement_string;
use std::collections::HashMap;

mod repl;

pub use repl::Repl;

/// Corresponds to Lua 5.3 `string.gsub`
pub fn gsub<'a>(
    text: &'a str,
    pattern: &str,
    repl: Repl<'a>,
    n: Option<usize>,
) -> Result<(String, usize)> {
    let text_bytes = text.as_bytes();
    let byte_len = text_bytes.len();

    let mut parser = Parser::new(pattern)?;
    let pattern_ast = parser.parse()?;

    let mut result = String::new();
    let mut last_pos = 0;
    let mut replacements = 0;
    let max_replacements = n.unwrap_or(usize::MAX);

    while replacements < max_replacements {
        match find_first_match(&pattern_ast, text_bytes, last_pos)? {
            Some((match_range, captures)) => {
                result.push_str(&text[last_pos..match_range.start]);

                let full_match = &text[match_range.start..match_range.end];
                let captures_str: Vec<&str> = captures
                    .iter()
                    .filter_map(|maybe_range| {
                        maybe_range
                            .as_ref()
                            .map(|range| &text[range.start..range.end])
                    })
                    .collect();

                match &repl {
                    Repl::String(repl_str) => {
                        let replacement = process_replacement_string(repl_str, &captures_str)?;
                        result.push_str(&replacement);
                    }
                    Repl::Function(f) => {
                        let mut args = Vec::with_capacity(captures_str.len() + 1);
                        args.push(full_match);
                        args.extend(captures_str.iter());
                        let replacement = f(&args);
                        result.push_str(&replacement);
                    }
                    Repl::Table(table) => {
                        let key = if !captures_str.is_empty() {
                            captures_str[0]
                        } else {
                            full_match
                        };

                        if let Some(replacement) = table.get(key) {
                            result.push_str(replacement);
                        } else {
                            result.push_str(full_match);
                        }
                    }
                }

                last_pos = match_range.end;
                replacements += 1;

                if match_range.start == match_range.end {
                    if last_pos >= byte_len {
                        break;
                    }
                    result.push_str(&text[last_pos..last_pos + 1]);
                    last_pos += 1;
                }
            }
            None => break,
        }
    }

    if last_pos < byte_len {
        result.push_str(&text[last_pos..]);
    }

    Ok((result, replacements))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_gsub() {
        assert_eq!(
            gsub("hello world", "l", Repl::String("L"), None),
            Ok(("heLLo worLd".to_string(), 3))
        );
        assert_eq!(
            gsub("hello world", "l", Repl::String("L"), Some(2)),
            Ok(("heLLo world".to_string(), 2))
        );
    }

    #[test]
    fn test_gsub_with_captures() {
        assert_eq!(
            gsub(
                "name=John age=25",
                "(%w+)=(%w+)",
                Repl::String("%2 is %1"),
                None
            ),
            Ok(("John is name 25 is age".to_string(), 2))
        );
    }

    #[test]
    fn test_gsub_with_patterns() {
        assert_eq!(
            gsub("hello 123 world 456", "%d+", Repl::String("<number>"), None),
            Ok(("hello <number> world <number>".to_string(), 2))
        );
    }

    #[test]
    fn test_gsub_with_empty_matches() {
        assert_eq!(
            gsub("hello", "", Repl::String("-"), None),
            Ok(("-h-e-l-l-o-".to_string(), 6))
        );
    }

    #[test]
    fn test_gsub_escape_percent() {
        assert_eq!(
            gsub("hello", "e", Repl::String("%% escaped"), None),
            Ok(("h% escapedllo".to_string(), 1))
        );
    }

    #[test]
    fn test_gsub_complex() {
        assert_eq!(
            gsub(
                "User: John, Age: 25, Email: john@example.com",
                "(User: )(%w+)(, Age: )(%d+)",
                Repl::String("%1%2%3%4 (adult)"),
                None
            ),
            Ok((
                "User: John, Age: 25 (adult), Email: john@example.com".to_string(),
                1
            ))
        );
    }

    #[test]
    fn test_gsub_with_function() {
        let result = gsub(
            "hello world",
            "%w+",
            Repl::Function(Box::new(|captures: &[&str]| {
                let word = captures[0];
                word.to_uppercase()
            })),
            None,
        );
        assert_eq!(result, Ok(("HELLO WORLD".to_string(), 2)));

        let result = gsub(
            "a=1, b=2, c=3",
            "(%w)=(%d)",
            Repl::Function(Box::new(|captures: &[&str]| {
                format!(
                    "{}={}",
                    captures[1],
                    captures[2].parse::<i32>().unwrap() * 2
                )
            })),
            None,
        );
        assert_eq!(result, Ok(("a=2, b=4, c=6".to_string(), 3)));
    }

    #[test]
    fn test_gsub_with_table() {
        let mut table = HashMap::new();
        table.insert("hello".to_string(), "привет".to_string());
        table.insert("world".to_string(), "мир".to_string());

        let result = gsub("hello world", "%w+", Repl::Table(&table), None);
        assert_eq!(result, Ok(("привет мир".to_string(), 2)));

        let mut table = HashMap::new();
        table.insert("hello".to_string(), "привет".to_string());

        let result = gsub("hello world", "%w+", Repl::Table(&table), None);
        assert_eq!(result, Ok(("привет world".to_string(), 2)));
    }

    #[test]
    fn test_gsub_with_captures_and_table() {
        let mut table = HashMap::new();
        table.insert("name".to_string(), "имя".to_string());
        table.insert("age".to_string(), "возраст".to_string());

        let result = gsub("name=John age=25", "(%w+)=%w+", Repl::Table(&table), None);
        assert_eq!(result, Ok(("имя возраст".to_string(), 2)));
    }
}
