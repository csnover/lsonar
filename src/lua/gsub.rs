use super::super::{Parser, Result, engine::find_first_match};
use repl::process_replacement_string;

mod repl;

pub use repl::Repl;

/// Corresponds to Lua 5.3 `string.gsub`
pub fn gsub<'a>(
    text: &'a [u8],
    pattern: &[u8],
    repl: Repl<'a>,
    n: Option<usize>,
) -> Result<(Vec<u8>, usize)> {
    let byte_len = text.len();

    let mut parser = Parser::new(pattern)?;
    let pattern_ast = parser.parse()?;

    let mut result = Vec::new();
    let mut last_pos = 0;
    let mut replacements = 0;
    let max_replacements = n.unwrap_or(usize::MAX);

    while replacements < max_replacements {
        match find_first_match(&pattern_ast, text, last_pos)? {
            Some((match_range, captures)) => {
                result.extend(&text[last_pos..match_range.start]);

                let full_match = &text[match_range.start..match_range.end];
                let captures_str: Vec<&[u8]> = captures
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
                        result.extend(&replacement);
                    }
                    Repl::Function(f) => {
                        let mut args = Vec::with_capacity(captures_str.len() + 1);
                        args.push(full_match);
                        args.extend(captures_str.iter());
                        let replacement = f(&args);
                        result.extend(&replacement);
                    }
                    Repl::Table(table) => {
                        let key = if !captures_str.is_empty() {
                            captures_str[0]
                        } else {
                            full_match
                        };

                        if let Some(replacement) = table.get(key) {
                            result.extend(*replacement);
                        } else {
                            result.extend(full_match);
                        }
                    }
                }

                last_pos = match_range.end;
                replacements += 1;

                if match_range.start == match_range.end {
                    if last_pos >= byte_len {
                        break;
                    }
                    result.extend(&text[last_pos..last_pos + 1]);
                    last_pos += 1;
                }
            }
            None => break,
        }
    }

    if last_pos < byte_len {
        result.extend(&text[last_pos..]);
    }

    Ok((result, replacements))
}
