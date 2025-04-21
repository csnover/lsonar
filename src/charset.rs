use super::{Error, Result};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CharSet {
    bytes: [bool; 256],
}

impl Default for CharSet {
    fn default() -> Self {
        Self::new()
    }
}

impl CharSet {
    pub const fn new() -> Self {
        CharSet {
            bytes: [false; 256],
        }
    }

    pub const fn full() -> Self {
        CharSet { bytes: [true; 256] }
    }

    pub fn add_byte(&mut self, b: u8) {
        self.bytes[b as usize] = true;
    }

    pub fn add_range(&mut self, start: u8, end: u8) -> Result<()> {
        if start > end {
            return Err(Error::Parser(
                "invalid range in character class".to_string(),
            ));
        }
        for b in start..=end {
            self.add_byte(b);
        }
        Ok(())
    }

    pub fn add_class(&mut self, class_char: char) -> Result<()> {
        match class_char {
            'a' => ('a'..='z')
                .chain('A'..='Z')
                .for_each(|c| self.add_byte(c as u8)),
            'c' => (0x00..=0x1f)
                .chain(std::iter::once(0x7f))
                .for_each(|b| self.add_byte(b)),
            'd' => ('0'..='9').for_each(|c| self.add_byte(c as u8)),
            'g' => (0x21..=0x7e).for_each(|b| self.add_byte(b)),
            'l' => ('a'..='z').for_each(|c| self.add_byte(c as u8)),
            'p' => (0x20..=0x7e).for_each(|b| {
                if !((b'a'..=b'z').contains(&b)
                    || (b'A'..=b'Z').contains(&b)
                    || (b'0'..=b'9').contains(&b))
                {
                    self.add_byte(b);
                }
            }),
            's' => [b' ', b'\t', b'\n', b'\r', 0x0b, 0x0c]
                .iter()
                .for_each(|&b| self.add_byte(b)),
            'u' => ('A'..='Z').for_each(|c| self.add_byte(c as u8)),
            'w' => ('a'..='z')
                .chain('A'..='Z')
                .chain('0'..='9')
                .for_each(|c| self.add_byte(c as u8)),
            'x' => ('0'..='9')
                .chain('a'..='f')
                .chain('A'..='F')
                .for_each(|c| self.add_byte(c as u8)),
            _ => {
                return Err(Error::Parser(format!(
                    "invalid character class '%{}'",
                    class_char
                )));
            }
        }
        Ok(())
    }

    #[inline]
    pub fn contains(&self, b: u8) -> bool {
        self.bytes[b as usize]
    }

    #[inline]
    pub fn invert(&mut self) {
        for i in 0..256 {
            self.bytes[i] = !self.bytes[i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty_full_charset() {
        let empty = CharSet::new();
        let full = CharSet::full();
        for i in 0..=255 {
            assert!(!empty.contains(i));
            assert!(full.contains(i));
        }
    }

    #[test]
    fn test_add_byte_charset() {
        let mut set = CharSet::new();
        assert!(!set.contains(b'a'));
        set.add_byte(b'a');
        assert!(set.contains(b'a'));
        assert!(!set.contains(b'b'));
    }

    #[test]
    fn test_add_range_charset() -> Result<()> {
        let mut set = CharSet::new();
        set.add_range(b'a', b'c')?;
        assert!(set.contains(b'a'));
        assert!(set.contains(b'b'));
        assert!(set.contains(b'c'));
        assert!(!set.contains(b'd'));
        Ok(())
    }

    #[test]
    fn test_add_range_invalid_charset() {
        let mut set = CharSet::new();
        assert!(matches!(set.add_range(b'z', b'a'), Err(Error::Parser(_))));
    }

    #[test]
    fn test_add_class_digit_charset() -> Result<()> {
        let mut set = CharSet::new();
        set.add_class('d')?;
        assert!(set.contains(b'0'));
        assert!(set.contains(b'5'));
        assert!(set.contains(b'9'));
        assert!(!set.contains(b'a'));
        assert!(!set.contains(b' '));
        Ok(())
    }

    #[test]
    fn test_add_class_space_charset() -> Result<()> {
        let mut set = CharSet::new();
        set.add_class('s')?;
        assert!(set.contains(b' '));
        assert!(set.contains(b'\t'));
        assert!(set.contains(b'\n'));
        assert!(set.contains(b'\r'));
        assert!(!set.contains(b'a'));
        Ok(())
    }

    #[test]
    fn test_add_class_invalid_charset() {
        let mut set = CharSet::new();
        assert!(matches!(set.add_class('Z'), Err(Error::Parser(_))));
        assert!(matches!(set.add_class('%'), Err(Error::Parser(_))));
    }

    #[test]
    fn test_contains_charset() {
        let mut set = CharSet::new();
        set.add_byte(b'x');
        assert!(set.contains(b'x'));
        assert!(!set.contains(b'y'));
    }

    #[test]
    fn test_invert_charset() {
        let mut set = CharSet::new();
        set.add_byte(b'a');
        set.add_byte(b'z');

        assert!(set.contains(b'a'));
        assert!(!set.contains(b'b'));
        assert!(set.contains(b'z'));

        set.invert();

        assert!(!set.contains(b'a'));
        assert!(set.contains(b'b'));
        assert!(!set.contains(b'z'));

        let mut full_set = CharSet::full();
        full_set.invert();
        let empty_set = CharSet::new();
        assert_eq!(full_set.bytes, empty_set.bytes);

        let mut empty_set_2 = CharSet::new();
        empty_set_2.invert();
        let full_set_2 = CharSet::full();
        assert_eq!(empty_set_2.bytes, full_set_2.bytes);
    }
}
