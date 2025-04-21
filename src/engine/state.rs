use std::{ops::Range, rc::Rc};

use crate::LUA_MAXCAPTURES;

#[derive(Clone)]
pub struct State {
    pub input: Rc<[u8]>,
    pub current_pos: usize,
    pub search_start_pos: usize,
    pub captures: Vec<Option<Range<usize>>>,
    pub recursion_depth: u32,
}

pub const MAX_RECURSION_DEPTH: u32 = 500;

impl State {
    pub fn new(input_slice: &[u8], start_pos: usize) -> Self {
        State {
            input: Rc::from(input_slice),
            current_pos: start_pos,
            search_start_pos: start_pos,
            captures: vec![None; LUA_MAXCAPTURES],
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
    pub fn check_class(&self, class_char: char, negated: bool) -> bool {
        if let Some(byte) = self.current_byte() {
            let matches = match class_char {
                'a' => byte.is_ascii_alphabetic(),
                'c' => byte.is_ascii_control(),
                'd' => byte.is_ascii_digit(),
                'g' => byte.is_ascii_graphic() && byte != b' ', // Lua's %g excludes space
                'l' => byte.is_ascii_lowercase(),
                'p' => byte.is_ascii_punctuation(),
                's' => byte.is_ascii_whitespace(),
                'u' => byte.is_ascii_uppercase(),
                'w' => byte.is_ascii_alphanumeric(),
                'x' => byte.is_ascii_hexdigit(),
                _ => false,
            };
            matches ^ negated // XOR handles negation
        } else {
            false
        }
    }
}