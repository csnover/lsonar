use super::{
    LUA_MAXCAPTURES,
    ast::{AstNode, AstRoot, Quantifier},
};
use std::{borrow::Cow, ops::Range};

#[cfg(test)]
mod tests;

/// Tries to find the first match of the pattern in the input string,
/// starting the search at `start_index` (0-based).
/// Returns the range of the full match and the ranges of captures if successful.
#[must_use]
pub fn find_first_match(
    pattern_ast: &AstRoot,
    input: &[u8],
    start_index: usize,
) -> Option<MatchRanges> {
    let input_len = input.len();

    if start_index > input_len {
        // Lua allows start > len for matching empty patterns at the end
        // Let the loop handle this. If start_index is way too large, it won't loop.
    }

    for i in start_index..=input_len {
        let initial_state = State::new(input, i);

        if let Some(final_state) = match_recursive(pattern_ast, initial_state) {
            let full_match_range = i..final_state.current_pos;
            return Some(MatchRanges {
                full_match: full_match_range,
                captures: final_state.captures[..pattern_ast.capture_count()].to_vec(),
            });
        }

        if let Some(AstNode::AnchorStart) = pattern_ast.first()
            && i == start_index
        {
            break;
        }
        // TODO: What is missing here? This is a no-op
        // if pattern_ast.len() == 1 {
        //     if let Some(AstNode::AnchorEnd) = pattern_ast.first() {
        //         if i < input_len {
        //             continue;
        //         }
        //     }
        // }
    }

    None
}

fn match_recursive<'a>(ast: &[AstNode], mut state: State<'a>) -> Option<State<'a>> {
    if state.recursion_depth > MAX_RECURSION_DEPTH {
        return None;
    }
    state.recursion_depth += 1;

    let Some((node, remaining_ast)) = ast.split_first() else {
        return Some(state);
    };

    match node {
        AstNode::Literal(b) => {
            if state.current_byte() == Some(*b) {
                state.current_pos += 1;
                match_recursive(remaining_ast, state)
            } else {
                None
            }
        }
        AstNode::Any => {
            if state.current_byte().is_some() {
                state.current_pos += 1;
                match_recursive(remaining_ast, state)
            } else {
                None
            }
        }
        AstNode::Class(c, negated) => {
            if state.check_class(*c, *negated) {
                state.current_pos += 1;
                match_recursive(remaining_ast, state)
            } else {
                None
            }
        }
        AstNode::Set(charset) => {
            if let Some(b) = state.current_byte()
                && charset.contains(b)
            {
                state.current_pos += 1;
                match_recursive(remaining_ast, state)
            } else {
                None
            }
        }
        AstNode::AnchorStart => {
            if state.current_pos == state.search_start_pos {
                match_recursive(remaining_ast, state)
            } else {
                None
            }
        }
        AstNode::AnchorEnd => {
            if state.current_pos == state.input.len() {
                match_recursive(remaining_ast, state)
            } else {
                None
            }
        }
        AstNode::Capture { index, inner } => {
            let start_pos = state.current_pos;
            let capture_index = *index - 1; // 0-based for [`Vec`] index

            if let Some(mut success_state) = match_recursive(inner, state.clone()) {
                success_state.captures[capture_index] = if inner.is_empty() {
                    CaptureRange::Position(start_pos)
                } else {
                    CaptureRange::Range(start_pos..success_state.current_pos)
                };

                if let Some(final_state) = match_recursive(remaining_ast, success_state) {
                    return Some(final_state);
                }
            }
            None
        }

        &AstNode::CaptureRef(index) => {
            // TODO: Error handling needs to be better. It is an error to use
            // index 0 or index > the total number of capture groups.
            match &state.captures[usize::from(index - 1)] {
                &CaptureRange::Position(pos) => state.current_pos == pos,
                CaptureRange::Range(range) => {
                    assert!(!range.is_empty());
                    let here = state.current_pos..(state.current_pos + range.len());
                    state.input.get(range.clone()) == state.input.get(here)
                }
            }
            .then_some(state)
        }

        AstNode::Balanced(b1, b2) => {
            if state.current_byte() != Some(*b1) {
                return None;
            }

            let mut balance = 1;
            let mut pos = state.current_pos + 1;
            while pos < state.input.len() {
                if state.input[pos] == *b2 {
                    balance -= 1;
                    if balance == 0 {
                        state.current_pos = pos + 1;
                        return match_recursive(remaining_ast, state);
                    }
                } else if state.input[pos] == *b1 {
                    balance += 1;
                }
                pos += 1;
            }
            None
        }

        AstNode::Frontier(charset) => {
            let prev_byte_in_set = charset.contains(state.previous_byte().unwrap_or(b'\0'));
            let next_byte_in_set = charset.contains(state.current_byte().unwrap_or(b'\0'));

            if !prev_byte_in_set && next_byte_in_set {
                match_recursive(remaining_ast, state)
            } else {
                None
            }
        }

        AstNode::Quantified { item, quantifier } => {
            match quantifier {
                Quantifier::Star | Quantifier::Plus => {
                    // Greedy *, +
                    let min_matches = usize::from(*quantifier == Quantifier::Plus);
                    match_greedy_quantifier(item.as_ref(), remaining_ast, state, min_matches)
                }
                Quantifier::Question => {
                    // Greedy ? (0 or 1)
                    let item_ast = std::slice::from_ref(item.as_ref());
                    if let Some(state_after_1) = match_recursive(item_ast, state.clone())
                        && let Some(final_state) =
                            match_recursive(remaining_ast, state_after_1.clone())
                    {
                        return Some(final_state.clone());
                    }
                    match_recursive(remaining_ast, state)
                }
                Quantifier::Minus => {
                    match_non_greedy_quantifier(item.as_ref(), remaining_ast, state)
                }
            }
        }
    }
}

fn match_greedy_quantifier<'a>(
    item: &AstNode,
    remaining_ast: &[AstNode],
    initial_state: State<'a>,
    min_matches: usize,
) -> Option<State<'a>> {
    let mut current_state = initial_state;
    let mut successful_match_states = Vec::new();

    for _ in 0..min_matches {
        if let Some(next_state) = match_recursive(std::slice::from_ref(item), current_state.clone())
            && next_state.current_pos != current_state.current_pos
        {
            current_state = next_state;
        } else {
            return None;
        }
    }
    successful_match_states.push(current_state.clone());

    while let Some(next_state) = match_recursive(std::slice::from_ref(item), current_state.clone())
    {
        if next_state.current_pos == current_state.current_pos {
            successful_match_states.push(next_state.clone());
        }
        current_state = next_state;
        successful_match_states.push(current_state.clone());
    }

    while !successful_match_states.is_empty() {
        let state_to_try = successful_match_states.pop()?;
        if let Some(final_state) = match_recursive(remaining_ast, state_to_try.clone()) {
            return Some(final_state.clone());
        }
    }

    None
}

fn match_non_greedy_quantifier<'a>(
    item: &AstNode,
    remaining_ast: &[AstNode],
    initial_state: State<'a>,
) -> Option<State<'a>> {
    let mut current_state = initial_state;

    loop {
        if let Some(final_state) = match_recursive(remaining_ast, current_state.clone()) {
            return Some(final_state.clone());
        }

        let item_ast = std::slice::from_ref(item);
        if let Some(next_state) = match_recursive(item_ast, current_state.clone()) {
            if next_state.current_pos == current_state.current_pos {
                if let Some(final_state) = match_recursive(remaining_ast, next_state.clone()) {
                    return Some(final_state.clone());
                }
                return None;
            }
            current_state = next_state;
        } else {
            return None;
        }
    }
}

#[derive(Clone)]
struct State<'a> {
    input: &'a [u8],
    current_pos: usize,
    search_start_pos: usize,
    captures: [CaptureRange; LUA_MAXCAPTURES],
    recursion_depth: u32,
}

const MAX_RECURSION_DEPTH: u32 = 500;

impl<'a> State<'a> {
    fn new(input_slice: &'a [u8], start_pos: usize) -> Self {
        Self {
            input: input_slice,
            current_pos: start_pos,
            search_start_pos: start_pos,
            captures: <_>::default(),
            recursion_depth: 0,
        }
    }

    #[inline]
    fn current_byte(&self) -> Option<u8> {
        self.input.get(self.current_pos).copied()
    }

    #[inline]
    fn previous_byte(&self) -> Option<u8> {
        self.current_pos
            .checked_sub(1)
            .and_then(|pos| self.input.get(pos).copied())
    }

    #[inline]
    fn check_class(&self, class_byte: u8, negated: bool) -> bool {
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
                b'z' => byte == 0,
                _ => false,
            };
            matches ^ negated // XOR handles negation
        } else {
            false
        }
    }
}

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

// TODO: This is only required for unit tests.
impl From<Range<usize>> for CaptureRange {
    fn from(value: Range<usize>) -> Self {
        Self::Range(value)
    }
}

// TODO: This is only required for unit tests.
impl PartialEq<Range<usize>> for CaptureRange {
    fn eq(&self, other: &Range<usize>) -> bool {
        match self {
            CaptureRange::Range(range) => range == other,
            CaptureRange::Position(_) => false,
        }
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

// TODO: This exists only to avoid having to spend a bunch of time changing the
// unit tests
impl PartialEq<(Range<usize>, Vec<CaptureRange>)> for MatchRanges {
    fn eq(&self, other: &(Range<usize>, Vec<CaptureRange>)) -> bool {
        self.full_match == other.0 && self.captures == other.1
    }
}
