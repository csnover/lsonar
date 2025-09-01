use std::{borrow::Cow, ops::Range};

use super::ast::{AstNode, AstRoot, Quantifier};
use state::{MAX_RECURSION_DEPTH, State};

mod state;

/// A capture group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Capture {
    /// A substring capture group.
    Range(Range<usize>),
    /// A current string position capture group.
    Position(usize),
}

impl Capture {
    #[must_use]
    pub fn into_bytes(self, text: &[u8]) -> Cow<'_, [u8]> {
        match self {
            Capture::Range(range) => Cow::Borrowed(&text[range]),
            Capture::Position(at) => Cow::Owned(
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

impl Default for Capture {
    fn default() -> Self {
        Self::Range(<_>::default())
    }
}

// TODO: This is only required for unit tests.
impl From<Range<usize>> for Capture {
    fn from(value: Range<usize>) -> Self {
        Self::Range(value)
    }
}

// TODO: This is only required for unit tests.
impl PartialEq<Range<usize>> for Capture {
    fn eq(&self, other: &Range<usize>) -> bool {
        match self {
            Capture::Range(range) => range == other,
            Capture::Position(_) => false,
        }
    }
}

/// The ranged indexes of a matched pattern. These are always 0-indexed.
#[derive(Debug, Eq, PartialEq)]
pub struct MatchRanges {
    /// The full range of the matched pattern.
    pub full_match: Range<usize>,
    /// The ranges of each captured group. If a group did not capture anything,
    /// the range will be empty.
    pub captures: Vec<Capture>,
}

// TODO: This exists only to avoid having to spend a bunch of time changing the
// unit tests
impl PartialEq<(Range<usize>, Vec<Capture>)> for MatchRanges {
    fn eq(&self, other: &(Range<usize>, Vec<Capture>)) -> bool {
        self.full_match == other.0 && self.captures == other.1
    }
}

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

    if ast.is_empty() {
        return Some(state);
    }

    let node = ast.first().unwrap();
    let remaining_ast = ast.get(1..).unwrap_or(&[]);

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
                    Capture::Position(start_pos)
                } else {
                    Capture::Range(start_pos..success_state.current_pos)
                };

                if let Some(final_state) = match_recursive(remaining_ast, success_state) {
                    return Some(final_state);
                }
            }
            None
        }

        AstNode::CaptureRef(_) => None,

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
            let prev_byte_in_set = state.previous_byte().is_some_and(|b| charset.contains(b));
            let next_byte_in_set = state.current_byte().is_some_and(|b| charset.contains(b));

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
