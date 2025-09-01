use crate::{Result, ast::parse_pattern};

mod iter;

pub use iter::GMatchIterator;

/// Like Lua
/// [`string.gmatch`](https://www.lua.org/manual/5.3/manual.html#pdf-string.gmatch),
/// returns an iterator of the captures of `pattern` over the string `s`.
///
/// # Errors
///
/// If the pattern string could not be parsed, an [`Error`](crate::Error) is returned.
///
/// # Feature flags
///
/// Captured string positions are 1-indexed if the `1-based` feature is enabled.
pub fn gmatch<'a>(s: &'a [u8], pattern: &[u8]) -> Result<GMatchIterator<'a>> {
    let is_empty_pattern = pattern.is_empty();

    let pattern_ast = if is_empty_pattern {
        <_>::default()
    } else {
        parse_pattern(pattern)?
    };

    Ok(GMatchIterator {
        bytes: s,
        pattern_ast,
        current_pos: 0,
        is_empty_pattern,
    })
}
