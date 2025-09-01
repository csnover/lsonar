use super::Capture;
use crate::LUA_MAXCAPTURES;

#[derive(Clone)]
pub struct State<'a> {
    pub input: &'a [u8],
    pub current_pos: usize,
    pub search_start_pos: usize,
    pub captures: [Capture; LUA_MAXCAPTURES],
    pub recursion_depth: u32,
}

pub const MAX_RECURSION_DEPTH: u32 = 500;

impl<'a> State<'a> {
    pub fn new(input_slice: &'a [u8], start_pos: usize) -> Self {
        Self {
            input: input_slice,
            current_pos: start_pos,
            search_start_pos: start_pos,
            captures: <_>::default(),
            recursion_depth: 0,
        }
    }

    #[inline]
    pub fn current_byte(&self) -> Option<u8> {
        self.input.get(self.current_pos).copied()
    }

    #[inline]
    pub fn previous_byte(&self) -> Option<u8> {
        if self.current_pos > 0 {
            self.input.get(self.current_pos - 1).copied()
        } else {
            None
        }
    }

    #[inline]
    pub fn check_class(&self, class_byte: u8, negated: bool) -> bool {
        if let Some(byte) = self.current_byte() {
            let matches = match class_byte {
                b'a' => byte.is_ascii_alphabetic(),
                b'c' => byte.is_ascii_control(),
                b'd' => byte.is_ascii_digit(),
                b'g' => byte.is_ascii_graphic() && byte != b' ', // Lua's %g excludes space
                b'l' => byte.is_ascii_lowercase(),
                b'p' => byte.is_ascii_punctuation(),
                b's' => byte.is_ascii_whitespace(),
                b'u' => byte.is_ascii_uppercase(),
                b'w' => byte.is_ascii_alphanumeric(),
                b'x' => byte.is_ascii_hexdigit(),
                _ => false,
            };
            matches ^ negated // XOR handles negation
        } else {
            false
        }
    }
}
