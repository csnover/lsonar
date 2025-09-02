//! Pattern string character set types.

use std::ops::Bound;

/// The error type used by [`CharSet`].
#[derive(Debug, Eq, thiserror::Error, PartialEq)]
pub enum Error {
    /// An invalid range was given.
    #[error("invalid range ({0} > {1})")]
    Range(u8, u8),
    /// An invalid byte class was given.
    #[error("invalid byte class '%{}'", _0.escape_ascii())]
    ByteClass(u8),
}

/// A character class set.
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
    /// Creates a new empty character set.
    #[must_use]
    pub const fn new() -> Self {
        CharSet {
            bytes: [false; 256],
        }
    }

    /// Adds the given byte to the character set.
    #[inline]
    pub const fn add_byte(&mut self, b: u8) {
        self.bytes[b as usize] = true;
    }

    /// Adds a range of bytes to the character set.
    ///
    /// # Errors
    ///
    /// If the end of the range is after the start of the range, an [`Error`] is
    /// returned.
    pub fn add_range(&mut self, start: u8, end: u8) -> Result<(), Error> {
        if start <= end {
            self.bytes[to_usize(start..=end)].fill(true);
            Ok(())
        } else {
            Err(Error::Range(start, end))
        }
    }

    #[inline]
    fn fill(&mut self, indexes: impl IntoIterator<Item = u8>, invert: bool) {
        let indexes = indexes.into_iter().map(usize::from);

        let mut on = invert;
        let mut index = 0;
        for next in indexes {
            if on {
                self.bytes[index..next].fill(true);
            }
            on = !on;
            index = next;
        }

        if on {
            self.bytes[index..256].fill(true);
        }
    }

    /// Adds a character class (e.g. `%a`) to the character set.
    ///
    /// # Errors
    ///
    /// If the given `class_byte` is not a valid character class identifier, an
    /// [`Error`] is returned.
    pub fn add_class(&mut self, class_byte: u8) -> Result<(), Error> {
        let invert = class_byte.is_ascii_uppercase();
        match class_byte.to_ascii_lowercase() {
            b'a' => {
                self.fill([b'A', b'Z' + 1, b'a', b'z' + 1], invert);
            }
            b'c' => {
                self.fill([0x00, 0x1f + 1, 0x7f, 0x7f + 1], invert);
            }
            b'd' => {
                self.fill([b'0', b'9' + 1], invert);
            }
            b'g' => {
                self.fill([0x21, 0x7e + 1], invert);
            }
            b'l' => {
                self.fill([b'a', b'z' + 1], invert);
            }
            b'p' => {
                self.fill(
                    [
                        b' ',
                        b'/' + 1,
                        b':',
                        b'@' + 1,
                        b'[',
                        b'`' + 1,
                        b'{',
                        b'~' + 1,
                    ],
                    invert,
                );
            }
            b's' => {
                self.fill([b'\t', b'\t' + 1, b'\n', b'\r' + 1, b' ', b' ' + 1], invert);
            }
            b'u' => {
                self.fill([b'A', b'Z' + 1], invert);
            }
            b'w' => {
                self.fill([b'0', b'9' + 1, b'A', b'Z' + 1, b'a', b'z' + 1], invert);
            }
            b'x' => {
                self.fill([b'0', b'9' + 1, b'A', b'F' + 1, b'a', b'f' + 1], invert);
            }
            _ => {
                return Err(Error::ByteClass(class_byte));
            }
        }

        Ok(())
    }

    /// Returns `true` if the character set contains the given byte.
    #[inline]
    #[must_use]
    pub const fn contains(&self, b: u8) -> bool {
        self.bytes[b as usize]
    }

    /// Inverts the character set.
    #[inline]
    pub fn invert(&mut self) {
        for i in 0..256 {
            self.bytes[i] = !self.bytes[i];
        }
    }
}

#[inline]
fn to_usize<R: std::ops::RangeBounds<u8>>(r: R) -> (Bound<usize>, Bound<usize>) {
    (
        r.start_bound().map(|n| usize::from(*n)),
        r.end_bound().map(|n| usize::from(*n)),
    )
}
