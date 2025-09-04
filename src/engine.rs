use super::{
    LUA_MAXCAPTURES, {Error, Result},
};
use std::{borrow::Cow, ops::Range};

/// A capture group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CaptureRange {
    /// A substring capture group.
    Range(Range<usize>),
    /// A current string position capture group.
    Position(usize),
}

impl CaptureRange {
    #[must_use]
    pub fn into_bytes(self, text: &[u8]) -> Cow<'_, [u8]> {
        match self {
            CaptureRange::Range(range) => Cow::Borrowed(&text[range]),
            CaptureRange::Position(at) => Cow::Owned(
                format!(
                    "{}",
                    if cfg!(feature = "1-based") {
                        at.saturating_add(1)
                    } else {
                        at
                    }
                )
                .into_bytes(),
            ),
        }
    }
}

impl Default for CaptureRange {
    fn default() -> Self {
        Self::Range(<_>::default())
    }
}

/// The ranged indexes of a matched pattern. These are always 0-indexed.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct MatchRanges {
    /// The full range of the matched pattern.
    pub full_match: Range<usize>,
    /// The ranges of each captured group. If a group did not capture anything,
    /// the range will be empty.
    pub captures: Vec<CaptureRange>,
}

/// Tries to find the first match of the pattern in the input string,
/// starting the search at `start_index` (0-based).
/// Returns the range of the full match and the ranges of captures if successful.
pub fn find_first_match(
    input: &[u8],
    pattern: &[u8],
    start_index: usize,
) -> Result<Option<MatchRanges>> {
    let input_len = input.len();
    let is_anchored = pattern.first().is_some_and(|c| *c == b'^');
    let pattern = if is_anchored { &pattern[1..] } else { pattern };

    for start in start_index..=input_len {
        let mut state = State {
            input,
            pattern,
            level: 0,
            depth: MAX_RECURSION_DEPTH,
            captures: <_>::default(),
        };

        if let Some(end) = next_match(&mut state, start, 0)? {
            let full_match = start..end;
            return Ok(Some(MatchRanges {
                full_match,
                captures: state
                    .captures
                    .into_iter()
                    .take(state.level)
                    .map(CaptureRange::try_from)
                    .collect::<Result<_, _>>()?,
            }));
        }

        if is_anchored {
            break;
        }
    }

    Ok(None)
}

/// The main pattern matching function.
fn next_match(state: &mut State<'_>, mut s: usize, mut p: usize) -> Result<Option<usize>> {
    if state.depth == 0 {
        return Err(Error::TooComplex { pos: p });
    }

    state.depth -= 1;

    // A loop is used to avoid unnecessary recursion. Because the matching
    // engine tracks recursion explicitly in order to abort pathological cases,
    // it is not enough to rely on the compiler to set up tail calls anyway.
    let s = loop {
        if p == state.pattern.len() {
            break Some(s);
        }

        // Special items: captures, anchors, balances, and frontiers
        match state.pattern[p] {
            b'(' => {
                // It is possible we are at the end of an invalid pattern here.
                let (p, is_position) = if state.pattern.get(p + 1).copied().unwrap_or(b'\0') == b')'
                {
                    (p + 2, true)
                } else {
                    (p + 1, false)
                };
                return state.start_capture(s, p, is_position);
            }
            b')' => return state.end_capture(s, p + 1),
            b'$' => {
                if p + 1 != state.pattern.len() {
                    // Literal '$' in pattern, not an anchor. Process it as a
                    // normal character by allowing code flow to continue.
                } else if s == state.input.len() {
                    // Anchor in pattern at the end of input.
                    break Some(s);
                } else {
                    // Anchor in pattern, but not at the end of input.
                    break None;
                }
            }
            b'%' => match state.pattern.get(p + 1).copied().unwrap_or(b'\0') {
                b'b' => {
                    if let Some(next) = state.match_balance(s, p + 2)? {
                        // Balance sub-match succeeded. Advance input and step
                        // to the next token.
                        s = next;
                        p += 4;
                        continue;
                    }

                    // Balanced did not match.
                    break None;
                }
                b'f' => {
                    // Advance pattern to parse the frontier set.
                    p += 2;
                    if state.pattern.get(p).copied().unwrap_or(b'\0') != b'[' {
                        return Err(Error::IncompleteFrontier { pos: p });
                    }
                    let p_after = state.class_end(p)?;

                    // Lua manual: “The beginning and end of the subject are
                    // handled as if they were the character '\0'.”
                    let first = if s == 0 { b'\0' } else { state.input[s - 1] };
                    let last = state.input.get(s).copied().unwrap_or(b'\0');

                    if !state.is_in_set(first, p, p_after - 1)
                        && state.is_in_set(last, p, p_after - 1)
                    {
                        // Matched; advance the pattern and continue.
                        p = p_after;
                        continue;
                    }

                    // Frontier did not match.
                    break None;
                }
                b'0'..=b'9' => {
                    if let Some(next) = state.match_capture(s, p, state.pattern[p + 1])? {
                        // Matched; advance the pattern and the input and
                        // continue.
                        s = next;
                        p += 2;
                        continue;
                    }

                    // Captured string did not match.
                    break None;
                }
                _ => {
                    // This is actually a single character class, so handle
                    // it below.
                }
            },
            _ => {
                // This is actually a normal character, so handle it below.
            }
        }

        // Normal characters and character classes
        let p_after = state.class_end(p)?;
        // It is possible the character class is at the end of the pattern.
        let quantifier = state.pattern.get(p_after).copied().unwrap_or(b'\0');
        if state.is_single_match(s, p, p_after) {
            match quantifier {
                b'?' => {
                    if let item @ Some(_) = next_match(state, s + 1, p_after + 1)? {
                        // Matched one item successfully
                        break item;
                    }

                    // Matched zero items successfully
                    p = p_after + 1;
                    continue;
                }
                b'+' | b'*' => {
                    // For '+', one item was already matched by `single_match`
                    s = if state.pattern[p_after] == b'+' {
                        s + 1
                    } else {
                        s
                    };

                    // Match zero or more, greedily
                    break state.max_expand(s, p, p_after)?;
                }
                b'-' => break state.min_expand(s, p, p_after)?,
                _ => {
                    // It was not a quantifier after all, but some other
                    // character literal that matched
                    s += 1;
                    p = p_after;
                    continue;
                }
            }
        }

        // Nothing matched. Is it OK?
        if [b'*', b'?', b'-'].contains(&quantifier) {
            p = p_after + 1;
            continue;
        }

        // No, it is not OK. This is a failure condition.
        break None;
    };

    state.depth += 1;
    Ok(s)
}

struct State<'a> {
    /// The input string to match.
    input: &'a [u8],
    /// The pattern to match.
    pattern: &'a [u8],
    /// Recursion depth of `full_match`.
    depth: usize,
    /// Number of capture groups.
    level: usize,
    /// Intermediate capture group states.
    captures: [CaptureState; LUA_MAXCAPTURES],
}

impl State<'_> {
    /// Matches a pattern balance item. If successful, returns the next position
    /// of the input.
    fn match_balance(&self, s: usize, p: usize) -> Result<Option<usize>> {
        if p >= self.pattern.len() - 1 {
            return Err(Error::MissingBalanceArgs { pos: p });
        }

        let open = self.pattern[p];
        // It is possible that we are at the end of the input.
        if self.input.get(s).copied().unwrap_or(b'\0') != open {
            return Ok(None);
        }

        let close = self.pattern[p + 1];
        let mut count = 1;

        for s in s + 1..self.input.len() {
            if self.input[s] == close {
                count -= 1;
                if count == 0 {
                    return Ok(Some(s + 1));
                }
            } else if self.input[s] == open {
                count += 1;
            }
        }

        Ok(None)
    }

    /// Matches the capture group at the given level to the input string.
    /// Returns the next position of the input string if successful.
    fn match_capture(&self, s: usize, p: usize, level: u8) -> Result<Option<usize>> {
        let range = self.check_capture(p, level)?;
        let end = s + range.len();
        Ok((self.input.get(range.clone()) == self.input.get(s..end)).then_some(end))
    }

    /// Takes as many pattern items as possible and then backs off until either
    /// the rest of the pattern matches or there are no more items to give back.
    /// If successful, returns the next position of the input.
    fn max_expand(&mut self, s: usize, p: usize, p_end: usize) -> Result<Option<usize>> {
        let mut i = 0;
        while self.is_single_match(s + i, p, p_end) {
            i += 1;
        }
        while i != usize::MAX {
            if let result @ Some(_) = next_match(self, s + i, p_end + 1)? {
                return Ok(result);
            }
            i = i.wrapping_sub(1);
        }
        Ok(None)
    }

    /// Takes the fewest number of items possible until the rest of the pattern
    /// starts to fail to match. If successful, returns the next position of the
    /// input.
    fn min_expand(&mut self, mut s: usize, p: usize, p_end: usize) -> Result<Option<usize>> {
        loop {
            if let result @ Some(_) = next_match(self, s, p_end + 1)? {
                break Ok(result);
            } else if self.is_single_match(s, p, p_end) {
                s += 1;
            } else {
                break Ok(None);
            }
        }
    }

    /// Starts a new capture group. Completes matching the input and returns its
    /// final position if successful.
    fn start_capture(&mut self, s: usize, p: usize, is_position: bool) -> Result<Option<usize>> {
        if self.level >= LUA_MAXCAPTURES {
            return Err(Error::TooManyCaptures { pos: p });
        }

        let slot = &mut self.captures[self.level];

        *slot = if is_position {
            CaptureState::Finished(CaptureRange::Position(s))
        } else {
            CaptureState::Pending { start: s }
        };

        self.level += 1;

        Ok(next_match(self, s, p)?.or_else(|| {
            self.level -= 1;
            None
        }))
    }

    /// Finalises a new capture group. Completes matching the input and returns
    /// its final position if successful.
    fn end_capture(&mut self, s: usize, p: usize) -> Result<Option<usize>> {
        let level = self.capture_to_close(p)?;
        self.captures[level].finish(s, p)?;

        Ok(next_match(self, s, p)?.or_else(|| {
            self.captures[level].revert();
            None
        }))
    }

    /// Returns the index of the highest pending capture group still needing
    /// finalising.
    fn capture_to_close(&self, p: usize) -> Result<usize> {
        for level in (0..self.level).rev() {
            if matches!(self.captures[level], CaptureState::Pending { .. }) {
                return Ok(level);
            }
        }
        Err(Error::InvalidPatternCapture { pos: p })
    }

    /// Ensures the given capture index belongs to a finished capture group and
    /// returns its range if so.
    fn check_capture(&self, p: usize, level: u8) -> Result<&Range<usize>> {
        let (level, oops) = level.overflowing_sub(b'1');
        let index = usize::from(level);
        if !oops
            && index < self.level
            && let CaptureState::Finished(CaptureRange::Range(range)) = &self.captures[index]
        {
            Ok(range)
        } else {
            Err(Error::InvalidCaptureIndex {
                index: usize::from(level.wrapping_add(1)),
                pos: p,
            })
        }
    }

    /// Finds the end of a character set. Returns the next position of the
    /// pattern, or an error if the pattern ends before the set is closed.
    fn class_end(&self, mut p: usize) -> Result<usize> {
        let c = self.pattern[p];
        p += 1;
        Ok(match c {
            b'%' => {
                if p == self.pattern.len() {
                    return Err(Error::EndsWithPercent { pos: p });
                }
                p + 1
            }
            b'[' => {
                // It is possible that we are at the end of the pattern.
                if self.pattern.get(p).copied().unwrap_or(b'\0') == b'^' {
                    p += 1;
                }

                loop {
                    if p == self.pattern.len() {
                        return Err(Error::EndsWithoutBracket { pos: p });
                    }
                    p += 1;
                    if self.pattern[p - 1] == b'%' && p < self.pattern.len() {
                        p += 1;
                    }
                    // It is possible that we are at the end of the pattern.
                    if self.pattern.get(p).copied().unwrap_or(b'\0') == b']' {
                        break;
                    }
                }

                p + 1
            }
            _ => p,
        })
    }

    /// Checks whether the input matches the pattern item at the given range.
    fn is_single_match(&self, s: usize, p_start: usize, p_end: usize) -> bool {
        let Some(c) = self.input.get(s).copied() else {
            return false;
        };
        match self.pattern[p_start] {
            b'.' => true,
            b'%' => match_class(c, self.pattern[p_start + 1]),
            b'[' => self.is_in_set(c, p_start, p_end - 1),
            _ => self.pattern[p_start] == c,
        }
    }

    /// Checks whether the given input character matches the character set
    /// at the given range.
    fn is_in_set(&self, c: u8, mut p: usize, p_end: usize) -> bool {
        let mut matched = true;
        if self.pattern[p + 1] == b'^' {
            matched = false;
            p += 1;
        }

        loop {
            p += 1;
            if p == p_end {
                break !matched;
            }

            if self.pattern[p] == b'%' {
                // %w
                p += 1;
                if match_class(c, self.pattern[p]) {
                    break matched;
                }
            } else if self.pattern[p + 1] == b'-' && p + 2 < p_end {
                // [a-z]
                p += 2;
                if self.pattern[p - 2] <= c && c <= self.pattern[p] {
                    break matched;
                }
            } else if self.pattern[p] == c {
                // Literal character
                break matched;
            }
        }
    }
}

/// Intermediate state representation of a capture group.
#[derive(Clone)]
enum CaptureState {
    /// The capture group is waiting to be closed.
    Pending { start: usize },
    /// The capture group is fully created.
    Finished(CaptureRange),
}

impl CaptureState {
    /// Finalise a ranged capture group.
    fn finish(&mut self, end: usize, p: usize) -> Result<()> {
        match self {
            CaptureState::Pending { start } => {
                *self = CaptureState::Finished(CaptureRange::Range(*start..end));
                Ok(())
            }
            CaptureState::Finished(..) => Err(Error::InvalidPatternCapture { pos: p }),
        }
    }

    /// Roll back a ranged capture group to a pending state.
    fn revert(&mut self) {
        if let CaptureState::Finished(CaptureRange::Range(range)) = self {
            *self = CaptureState::Pending { start: range.start }
        }
    }
}

impl Default for CaptureState {
    fn default() -> Self {
        Self::Finished(<_>::default())
    }
}

impl TryFrom<CaptureState> for CaptureRange {
    type Error = Error;

    fn try_from(value: CaptureState) -> Result<Self, Self::Error> {
        match value {
            CaptureState::Pending { start } => Err(Error::UnfinishedCapture { pos: start }),
            CaptureState::Finished(capture_range) => Ok(capture_range),
        }
    }
}

const MAX_RECURSION_DEPTH: usize = 500;

const fn match_class(c: u8, class: u8) -> bool {
    let matches = match class.to_ascii_lowercase() {
        b'a' => c.is_ascii_alphabetic(),
        b'c' => c.is_ascii_control(),
        b'd' => c.is_ascii_digit(),
        b'g' => c.is_ascii_graphic(),
        b'l' => c.is_ascii_lowercase(),
        b'p' => c.is_ascii_punctuation(),
        b's' => c.is_ascii_whitespace(),
        b'u' => c.is_ascii_uppercase(),
        b'w' => c.is_ascii_alphanumeric(),
        b'x' => c.is_ascii_hexdigit(),
        b'z' => c == 0,
        _ => return c == class,
    };
    if class.is_ascii_lowercase() {
        matches
    } else {
        !matches
    }
}
