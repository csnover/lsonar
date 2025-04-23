use super::{
    super::{AstNode, Parser, Result, engine::find_first_match},
    calculate_start_index,
};
use std::ops::Range;

/// Corresponds to Lua 5.3 `string.gmatch`
pub fn gmatch<'a>(
    text: &'a str,
    pattern: &str,
) -> Result<impl Iterator<Item = Result<Vec<String>>> + 'a> {
    // Check for empty pattern
    let is_empty_pattern = pattern.is_empty();

    let pattern_ast = if is_empty_pattern {
        Vec::new()
    } else {
        let mut parser = Parser::new(pattern)?;
        parser.parse()?
    };

    Ok(GMatchIterator {
        text,
        text_bytes: text.as_bytes(),
        pattern_ast,
        current_pos: 0,
        is_empty_pattern,
    })
}

pub struct GMatchIterator<'a> {
    text: &'a str,
    text_bytes: &'a [u8],
    pattern_ast: Vec<AstNode>,
    current_pos: usize,
    is_empty_pattern: bool,
}

impl Iterator for GMatchIterator<'_> {
    type Item = Result<Vec<String>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pos > self.text_bytes.len() {
            return None;
        }

        if self.is_empty_pattern {
            let result = Some(Ok(vec![String::new()]));
            
            self.current_pos += 1;
            
            return result;
        }

        match find_first_match(&self.pattern_ast, self.text_bytes, self.current_pos) {
            Ok(Some((match_range, captures))) => {
                if match_range.start == match_range.end {
                    self.current_pos = match_range.end + 1;
                    if self.current_pos > self.text_bytes.len() {
                        return None;
                    }
                } else {
                    self.current_pos = match_range.end;
                }

                let result: Vec<String> = if captures.iter().any(|c| c.is_some()) {
                    captures
                        .into_iter()
                        .filter_map(|maybe_range| {
                            maybe_range.map(|range| {
                                String::from_utf8_lossy(&self.text_bytes[range]).into_owned()
                            })
                        })
                        .collect()
                } else {
                    vec![
                        String::from_utf8_lossy(
                            &self.text_bytes[match_range.start..match_range.end],
                        )
                        .into_owned(),
                    ]
                };

                Some(Ok(result))
            }
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn svec(items: &[&str]) -> Vec<String> {
        items.iter().map(|&s| s.to_string()).collect()
    }

    fn collect_gmatch(text: &str, pattern: &str) -> Result<Vec<Vec<String>>> {
        let it = gmatch(text, pattern)?;
        it.collect()
    }

    #[test]
    fn test_basic_gmatch() {
        assert_eq!(
            collect_gmatch("hello world", "hello"),
            Ok(vec![svec(&["hello"])])
        );
        assert_eq!(
            collect_gmatch("hello hello", "hello"),
            Ok(vec![svec(&["hello"]), svec(&["hello"])])
        );
        assert_eq!(
            collect_gmatch("abc123def456", "%d+"),
            Ok(vec![svec(&["123"]), svec(&["456"])])
        );
    }

    #[test]
    fn test_gmatch_with_captures() {
        assert_eq!(
            collect_gmatch("name=John age=25", "(%a+)=(%w+)"),
            Ok(vec![svec(&["name", "John"]), svec(&["age", "25"])])
        );
        assert_eq!(
            collect_gmatch("a=1 b=2 c=3", "(%a)=(%d)"),
            Ok(vec![
                svec(&["a", "1"]),
                svec(&["b", "2"]),
                svec(&["c", "3"])
            ])
        );
    }

    #[test]
    fn test_gmatch_with_empty_matches() {
        assert_eq!(collect_gmatch("abc", "()a()"), Ok(vec![svec(&["", ""])]));
        let result = collect_gmatch("abc", "").unwrap();
        for r in &result {
            assert_eq!(r, &svec(&[""]));
        }
    }

    #[test]
    fn test_gmatch_complex_patterns() {
        assert_eq!(
            collect_gmatch(
                "IPv4: 192.168.1.1 and 10.0.0.1",
                "(%d+)%.(%d+)%.(%d+)%.(%d+)"
            ),
            Ok(vec![
                svec(&["192", "168", "1", "1"]),
                svec(&["10", "0", "0", "1"])
            ])
        );

        assert_eq!(
            collect_gmatch("<p>First</p><p>Second</p>", "<p>([^<]+)</p>"),
            Ok(vec![svec(&["First"]), svec(&["Second"])])
        );
    }
}
