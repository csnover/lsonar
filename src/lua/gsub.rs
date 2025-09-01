use crate::{Result, ast::parse_pattern, engine::find_first_match};
use repl::process_replacement_string;

mod repl;

use crate::engine::MatchRanges;
pub use repl::Repl;
use std::borrow::Cow;

/// Like Lua
/// [`string.gsub`](https://www.lua.org/manual/5.3/manual.html#pdf-string.gsub),
/// returns a copy of `s` in which all (or the first `n`, if given) occurrences
/// of `pattern` are replaced by `repl`.
///
/// # Errors
///
/// If the pattern string could not be parsed, an [`Error`](crate::Error) is returned.
pub fn gsub<'a>(
    s: &'a [u8],
    pattern: &[u8],
    repl: Repl<'a>,
    n: Option<usize>,
) -> Result<(Vec<u8>, usize)> {
    let byte_len = s.len();

    let pattern_ast = parse_pattern(pattern)?;

    let mut result = Vec::new();
    let mut last_pos = 0;
    let mut replacements = 0;
    let max_replacements = n.unwrap_or(usize::MAX);

    while replacements < max_replacements
        && let Some(MatchRanges {
            full_match: match_range,
            captures,
        }) = find_first_match(&pattern_ast, s, last_pos)
    {
        result.extend(&s[last_pos..match_range.start]);

        let full_match = &s[match_range.clone()];

        match repl {
            Repl::String(repl_str) => {
                let captures_str = captures
                    .into_iter()
                    .map(|range| range.into_bytes(s))
                    .collect::<Vec<_>>();
                let replacement = process_replacement_string(repl_str, &captures_str);
                result.extend(&replacement);
            }
            Repl::Function(f) => {
                let args = core::iter::once(Cow::Borrowed(full_match))
                    .chain(captures.into_iter().map(|range| range.into_bytes(s)))
                    .collect::<Vec<_>>();
                let replacement = f(&args);
                result.extend(&replacement);
            }
            Repl::Table(f) => {
                let key = captures.first().map_or(Cow::Borrowed(full_match), |range| {
                    range.clone().into_bytes(s)
                });

                if let Some(replacement) = f(key) {
                    result.extend(replacement);
                } else {
                    result.extend(full_match);
                }
            }
        }

        last_pos = match_range.end;
        replacements += 1;

        if match_range.is_empty() {
            if last_pos >= byte_len {
                break;
            }
            result.extend(&s[last_pos..=last_pos]);
            last_pos += 1;
        }
    }

    if last_pos < byte_len {
        result.extend(&s[last_pos..]);
    }

    Ok((result, replacements))
}
