use crate::Result;

/// Corresponds to Lua 5.3 `string.gsub`
pub fn gsub(
    text: &str,
    pattern: &str,
    repl: &str, /* TODO: Support function/table */
    n: Option<usize>,
) -> Result<(String, usize)> {
    unimplemented!()
}