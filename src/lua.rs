pub mod find;
pub mod r#match;
pub mod gmatch;
pub mod gsub;

pub use self::find::find;

/// Calculates the 0-based Rust index from a 1-based Lua index.
/// Handles positive, negative (including 0 treated as 1), and default (1) indices according to Lua rules.
fn calculate_start_index(init: Option<isize>, text_len: usize) -> usize {
    match init {
        Some(i) => {
            if i > 0 {
                 (i as usize).saturating_sub(1).min(text_len)
            } else if i == 0 {
                 0
            } else {
                 text_len.saturating_add_signed(i).max(0)
            }
        }
        None => 0,
    }
}


#[cfg(test)]
mod tests {
    use super::calculate_start_index;

    #[test]
    fn test_calculate_start_index() {
        let text = "hello world";
        let len = text.chars().count();

        assert_eq!(calculate_start_index(None, len), 0);
        assert_eq!(calculate_start_index(Some(1), len), 0);
        assert_eq!(calculate_start_index(Some(5), len), 4);
        assert_eq!(calculate_start_index(Some(11), len), 10);
        assert_eq!(calculate_start_index(Some(12), len), 11);
        assert_eq!(calculate_start_index(Some(100), len), 11);

        assert_eq!(calculate_start_index(Some(0), len), 0);
        assert_eq!(calculate_start_index(Some(-1), len), 10);
        assert_eq!(calculate_start_index(Some(-5), len), 6);
        assert_eq!(calculate_start_index(Some(-11), len), 0);
        assert_eq!(calculate_start_index(Some(-12), len), 0);
        assert_eq!(calculate_start_index(Some(-100), len), 0);

        const EMPTY_LEN: usize = 0;
        assert_eq!(calculate_start_index(None, EMPTY_LEN), 0);
        assert_eq!(calculate_start_index(Some(1), EMPTY_LEN), 0);
        assert_eq!(calculate_start_index(Some(5), EMPTY_LEN), 0);
        assert_eq!(calculate_start_index(Some(0), EMPTY_LEN), 0);
        assert_eq!(calculate_start_index(Some(-1), EMPTY_LEN), 0);
        assert_eq!(calculate_start_index(Some(-5), EMPTY_LEN), 0);

        let utf_text = "你好世界";
        let utf_len = utf_text.chars().count();
        assert_eq!(calculate_start_index(Some(1), utf_len), 0);
        assert_eq!(calculate_start_index(Some(4), utf_len), 3);
        assert_eq!(calculate_start_index(Some(5), utf_len), 4);
        assert_eq!(calculate_start_index(Some(0), utf_len), 0);
        assert_eq!(calculate_start_index(Some(-1), utf_len), 3);
        assert_eq!(calculate_start_index(Some(-4), utf_len), 0);
        assert_eq!(calculate_start_index(Some(-5), utf_len), 0);
    }
}