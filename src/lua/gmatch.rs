use crate::{Parser, Result};

mod iter;

pub use iter::GMatchIterator;

/// Corresponds to Lua 5.3 `string.gmatch`
pub fn gmatch(text: &[u8], pattern: &[u8]) -> Result<GMatchIterator> {
    let is_empty_pattern = pattern.is_empty();

    let pattern_ast = if is_empty_pattern {
        Vec::new()
    } else {
        let mut parser = Parser::new(pattern)?;
        parser.parse()?
    };

    Ok(GMatchIterator {
        bytes: text.to_vec(),
        pattern_ast,
        current_pos: 0,
        is_empty_pattern,
    })
}
