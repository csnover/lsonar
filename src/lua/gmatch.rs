use crate::{Parser, Result};

mod iter;

pub use iter::GMatchIterator;

/// Corresponds to Lua 5.3 `string.gmatch`
pub fn gmatch<'a>(text: &'a [u8], pattern: &[u8]) -> Result<GMatchIterator<'a>> {
    let is_empty_pattern = pattern.is_empty();

    let pattern_ast = if is_empty_pattern {
        <_>::default()
    } else {
        let mut parser = Parser::new(pattern)?;
        parser.parse()?
    };

    Ok(GMatchIterator {
        bytes: text,
        pattern_ast,
        current_pos: 0,
        is_empty_pattern,
    })
}
