use std::ops::Range;

use super::{
    Result,
    ast::{AstNode, Quantifier},
};
use state::{MAX_RECURSION_DEPTH, State};

mod state;

/// Tries to find the first match of the pattern in the input string,
/// starting the search at `start_index` (0-based).
/// Returns the range of the full match and the ranges of captures if successful.
pub fn find_first_match(
    pattern_ast: &[AstNode],
    input: &[u8],
    start_index: usize,
) -> Result<Option<(Range<usize>, Vec<Option<Range<usize>>>)>> {
    let input_len = input.len();

    if start_index > input_len {
        // Lua allows start > len for matching empty patterns at the end
        // Let the loop handle this. If start_index is way too large, it won't loop.
    }

    for i in start_index..=input_len {
        let initial_state = State::new(input, i);

        if let Some(final_state) = match_recursive(pattern_ast, initial_state) {
            let full_match_range = i..final_state.current_pos;
            return Ok(Some((full_match_range, final_state.captures)));
        }

        if let Some(AstNode::AnchorStart) = pattern_ast.first() {
            if i == start_index {
                break;
            }
        }
        if pattern_ast.len() == 1 {
            if let Some(AstNode::AnchorEnd) = pattern_ast.first() {
                if i < input_len {
                    continue;
                }
            }
        }
    }

    Ok(None)
}

fn match_recursive(ast: &[AstNode], mut state: State) -> Option<State> {
    println!(
        "[{: >2}] match: pos={}, ast_len={}",
        state.recursion_depth,
        state.current_pos,
        ast.len()
    );
    if state.recursion_depth > MAX_RECURSION_DEPTH {
        println!(
            "[{: >2}] -> Max recursion depth exceeded, returning None",
            state.recursion_depth
        );
        return None;
    }
    state.recursion_depth += 1;

    if ast.is_empty() {
        println!(
            "[{: >2}] -> AST empty, returning Some(pos={})",
            state.recursion_depth - 1,
            state.current_pos
        );
        return Some(state);
    }

    let node = ast.first().unwrap();
    let remaining_ast = ast.get(1..).unwrap_or(&[]);
    println!(
        "[{: >2}]   Node: {:?}, Current char: {:?}",
        state.recursion_depth - 1,
        node,
        state.current_byte().map(|b| b as char)
    );

    let result = match node {
        AstNode::Literal(b) => {
            println!(
                "[{: >2}]   Matching Literal: {}",
                state.recursion_depth - 1,
                *b as char
            );
            if state.current_byte() == Some(*b) {
                state.current_pos += 1;
                println!(
                    "[{: >2}]     Literal matched, recursing for remaining...",
                    state.recursion_depth - 1
                );
                match_recursive(remaining_ast, state)
            } else {
                println!("[{: >2}]     Literal mismatch.", state.recursion_depth - 1);
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
            println!(
                "[{: >2}]   Matching Class: {} (negated={})",
                state.recursion_depth - 1,
                *c as char,
                *negated
            );
            if state.check_class(*c, *negated) {
                state.current_pos += 1;
                println!(
                    "[{: >2}]     Class matched, recursing for remaining...",
                    state.recursion_depth - 1
                );
                match_recursive(remaining_ast, state)
            } else {
                println!("[{: >2}]     Class mismatch.", state.recursion_depth - 1);
                None
            }
        }
        AstNode::Set(charset) => {
            if let Some(b) = state.current_byte() {
                if charset.contains(b) {
                    state.current_pos += 1;
                    match_recursive(remaining_ast, state)
                } else {
                    None
                }
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
            println!(
                "[{: >2}]   Entering Capture #{} (start_pos={})",
                state.recursion_depth - 1,
                index,
                state.current_pos
            );
            let start_pos = state.current_pos;
            let capture_index = *index - 1; // 0-based for [`Vec`] index

            println!(
                "[{: >2}]     Recursing for inner capture content...",
                state.recursion_depth - 1
            );
            if let Some(mut success_state) = match_recursive(inner, state.clone()) {
                let capture_range = start_pos..success_state.current_pos;
                println!(
                    "[{: >2}]     Inner capture match SUCCESS, range={:?}, new_pos={}",
                    state.recursion_depth - 1,
                    capture_range,
                    success_state.current_pos
                );
                success_state.captures[capture_index] = Some(capture_range.clone());

                println!(
                    "[{: >2}]     Recursing for remaining pattern after capture...",
                    state.recursion_depth - 1
                );
                if let Some(final_state) = match_recursive(remaining_ast, success_state) {
                    println!(
                        "[{: >2}]   Capture #{} returning Some(final_state)",
                        state.recursion_depth - 1,
                        index
                    );
                    return Some(final_state);
                } else {
                    println!(
                        "[{: >2}]     Remaining pattern after capture FAILED.",
                        state.recursion_depth - 1
                    );
                }
            } else {
                println!(
                    "[{: >2}]     Inner capture match FAILED.",
                    state.recursion_depth - 1
                );
            }
            println!(
                "[{: >2}]   Capture #{} returning None",
                state.recursion_depth - 1,
                index
            );
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
                    let min_matches = if *quantifier == Quantifier::Plus {
                        1
                    } else {
                        0
                    };
                    match_greedy_quantifier(item.as_ref(), remaining_ast, state, min_matches)
                }
                Quantifier::Question => {
                    // Greedy ? (0 or 1)
                    let item_ast = std::slice::from_ref(item.as_ref());
                    if let Some(state_after_1) = match_recursive(item_ast, state.clone()) {
                        if let Some(final_state) =
                            match_recursive(remaining_ast, state_after_1.clone())
                        {
                            return Some(final_state.clone());
                        }
                    }
                    match_recursive(remaining_ast, state)
                }
                Quantifier::Minus => {
                    match_non_greedy_quantifier(item.as_ref(), remaining_ast, state)
                }
            }
        }
    };
    result
}

fn match_greedy_quantifier(
    item: &AstNode,
    remaining_ast: &[AstNode],
    initial_state: State,
    min_matches: usize,
) -> Option<State> {
    let mut current_state = initial_state;
    let mut successful_match_states = Vec::new();

    for _ in 0..min_matches {
        if let Some(next_state) = match_recursive(std::slice::from_ref(item), current_state.clone())
        {
            if next_state.current_pos == current_state.current_pos {
                return None;
            }
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

fn match_non_greedy_quantifier(
    item: &AstNode,
    remaining_ast: &[AstNode],
    initial_state: State,
) -> Option<State> {
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
                } else {
                    return None;
                }
            }
            current_state = next_state;
        } else {
            return None;
        }
    }
}
