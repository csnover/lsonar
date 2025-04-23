use crate::Result;

/// Corresponds to Lua 5.3 `string.gmatch`
// Placeholder for the iterator type
struct GMatchIteratorPlaceholder;
impl Iterator for GMatchIteratorPlaceholder {
    type Item = Result<Vec<String>>;
    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

pub fn gmatch(text: &str, pattern: &str) -> Result<impl Iterator<Item = Result<Vec<String>>>> {
    // TODO
    Ok(GMatchIteratorPlaceholder)
}
