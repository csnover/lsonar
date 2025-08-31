pub mod find;
pub mod gmatch;
pub mod gsub;
pub mod r#match;

pub use self::{
    find::find,
    gmatch::gmatch,
    gsub::{Repl, gsub},
    r#match::r#match,
};

pub type Captures<'a> = Vec<&'a [u8]>;

fn calculate_start_index(text_len: usize, init: Option<isize>) -> usize {
    match init {
        Some(i) if i > 0 => {
            let i = if cfg!(feature = "1-based") { i - 1 } else { i };
            // Clippy: Precondition `i > 0` guarantees no sign loss
            #[allow(clippy::cast_sign_loss)]
            let i = i as usize;
            if i >= text_len { text_len } else { i }
        }
        Some(i) if i < 0 => {
            let abs_i = i.unsigned_abs();
            if abs_i > text_len {
                0
            } else {
                text_len.saturating_sub(abs_i)
            }
        }
        _ => 0,
    }
}
