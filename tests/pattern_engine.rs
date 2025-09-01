#![allow(clippy::single_range_in_vec_init)]

use lsonar::{
    Parser, Result,
    engine::{Capture, MatchRanges, find_first_match},
};
use std::ops::Range;

fn find(pattern_str: &[u8], text: &[u8]) -> Result<Option<MatchRanges>> {
    let mut parser = Parser::new(pattern_str)?;
    let ast = parser.parse()?;
    Ok(find_first_match(&ast, text, 0)) // 0-based index only for tests
}

#[track_caller]
fn assert_match(
    pattern: &[u8],
    text: &[u8],
    expected_full: Range<usize>,
    expected_captures: &[Capture],
) {
    let result = find(pattern, text).expect("find failed");
    match result {
        Some(MatchRanges {
            full_match,
            captures,
        }) => {
            assert_eq!(full_match, expected_full, "Full match range mismatch");
            assert_eq!(&captures[..], expected_captures, "Captures mismatch");
        }
        None => panic!(
            "Expected match, but found none for pattern '{}' in text '{}'",
            str::from_utf8(pattern).unwrap(),
            str::from_utf8(text).unwrap()
        ),
    }
}

#[track_caller]
fn assert_no_match(pattern: &[u8], text: &[u8]) {
    let result = find(pattern, text).expect("find failed");
    assert!(
        result.is_none(),
        "Expected no match, but found one for pattern '{}' in text '{}'",
        str::from_utf8(pattern).unwrap(),
        str::from_utf8(text).unwrap()
    );
}

#[test]
fn test_literal_match_engine() {
    assert_match(b"abc", b"abc", 0..3, &[]);
    assert_match(b"abc", b"xabc", 1..4, &[]);
    assert_match(b"abc", b"abcy", 0..3, &[]);
    assert_no_match(b"abc", b"axbyc");
    assert_no_match(b"abc", b"ab");
    assert_no_match(b"abc", b"");
}

#[test]
fn test_any_match_engine() {
    assert_match(b".", b"a", 0..1, &[]);
    assert_match(b"a.c", b"axc", 0..3, &[]);
    assert_match(b"a.c", b"a\nc", 0..3, &[]);
    assert_no_match(b".", b"");
}

#[test]
fn test_class_match_engine() {
    assert_match(b"%d", b"5", 0..1, &[]);
    assert_match(b"%a", b"Z", 0..1, &[]);
    assert_match(b"%l", b"z", 0..1, &[]);
    assert_match(b"%s", b" ", 0..1, &[]);
    assert_match(b"%x", b"f", 0..1, &[]);
    assert_match(b"a%dz", b"a1z", 0..3, &[]);
    assert_no_match(b"%d", b"a");
    assert_match(b"%D", b"a", 0..1, &[]);
    assert_no_match(b"%D", b"5");
    assert_match(b"%S", b"a", 0..1, &[]);
    assert_no_match(b"%S", b" ");
}

#[test]
fn test_set_match_engine() {
    assert_match(b"[abc]", b"a", 0..1, &[]);
    assert_match(b"[abc]", b"b", 0..1, &[]);
    assert_match(b"[^abc]", b"d", 0..1, &[]);
    assert_match(b"[a-z]", b"m", 0..1, &[]);
    assert_match(b"[%d%s]", b"5", 0..1, &[]);
    assert_match(b"[%d%s]", b" ", 0..1, &[]);
    assert_no_match(b"[abc]", b"d");
    assert_no_match(b"[^abc]", b"a");
    assert_no_match(b"[a-z]", b"A");
    assert_no_match(b"[a-z]", b"5");
    assert_no_match(b"[%d%s]", b"a");
}

#[test]
fn test_anchor_match_engine() {
    assert_match(b"^abc", b"abc", 0..3, &[]);
    assert_no_match(b"^abc", b"xabc");
    assert_match(b"abc$", b"abc", 0..3, &[]);
    assert_no_match(b"abc$", b"abcd");
    assert_match(b"^abc$", b"abc", 0..3, &[]);
    assert_no_match(b"^abc$", b"xabc");
    assert_no_match(b"^abc$", b"abcd");
    assert_match(b"^", b"", 0..0, &[]);
    assert_match(b"$", b"", 0..0, &[]);
    assert_match(b"^$", b"", 0..0, &[]);
}

#[test]
fn test_greedy_quantifiers_engine() {
    assert_match(b"a*", b"aaa", 0..3, &[]);
    assert_match(b"a*", b"", 0..0, &[]);
    assert_match(b"a*b", b"aaab", 0..4, &[]);
    assert_match(b"a*b", b"b", 0..1, &[]);
    assert_match(b"x*", b"y", 0..0, &[]);
    assert_match(b"a+", b"aaa", 0..3, &[]);
    assert_no_match(b"a+", b"");
    assert_match(b"a+b", b"aaab", 0..4, &[]);
    assert_no_match(b"a+b", b"b");
    assert_match(b"a?", b"a", 0..1, &[]);
    assert_match(b"a?", b"", 0..0, &[]);
    assert_match(b"a?b", b"ab", 0..2, &[]);
    assert_match(b"a?b", b"b", 0..1, &[]);
    assert_match(b"a*a", b"aaa", 0..3, &[]);
    assert_match(b".*b", b"axbyb", 0..5, &[]);
    assert_match(b"a+a", b"aa", 0..2, &[]);
    assert_match(b"a?a", b"aa", 0..2, &[]);
    assert_match(b"a?a", b"a", 0..1, &[]);
}

#[test]
fn test_non_greedy_quantifier_engine() {
    assert_match(b"a-", b"aaa", 0..0, &[]);
    assert_match(b"a-", b"", 0..0, &[]);
    assert_match(b"a-b", b"aaab", 0..4, &[]);
    assert_match(b"a-b", b"b", 0..1, &[]);
    assert_match(b"x-", b"y", 0..0, &[]);
    assert_match(b".-b", b"axbyb", 0..3, &[]);
    assert_match(b"a-a", b"aaa", 0..1, &[]);
}

#[test]
fn test_captures_simple_engine() {
    assert_match(b"(a)", b"a", 0..1, &[(0..1).into()]);
    assert_match(b"(.)", b"b", 0..1, &[(0..1).into()]);
    assert_match(b"(%d)", b"3", 0..1, &[(0..1).into()]);
    assert_match(b"a(b)c", b"abc", 0..3, &[(1..2).into()]);
    assert_match(b"a(.)c", b"axc", 0..3, &[(1..2).into()]);
    assert_match(b"(a)(b)", b"ab", 0..2, &[(0..1).into(), (1..2).into()]);
    assert_match(
        b"()(b)",
        b"b",
        0..1,
        &[Capture::Position(0), Capture::Range(0..1)],
    );
}

#[test]
fn test_captures_quantified_engine() {
    assert_match(b"(a)*", b"aaa", 0..3, &[(2..3).into()]);
    assert_match(b"(a)+", b"aaa", 0..3, &[(2..3).into()]);
    assert_match(b"(a)?", b"a", 0..1, &[(0..1).into()]);
    assert_match(b"(a)?", b"", 0..0, &[<_>::default()]);
    assert_match(b"a(b)*c", b"abbbc", 0..5, &[(3..4).into()]);
    assert_match(b"a(b)+c", b"abbbc", 0..5, &[(3..4).into()]);
    assert_match(b"a(b)?c", b"abc", 0..3, &[(1..2).into()]);
    assert_match(b"a(b)?c", b"ac", 0..2, &[<_>::default()]);
    assert_match(b"a(b)-c", b"abbbc", 0..5, &[(3..4).into()]);
    assert_match(b"a(b)-c", b"abbbc", 0..5, &[(3..4).into()]);
}

#[test]
fn test_captures_nested_engine() {
    assert_match(b"(a(b)c)", b"abc", 0..3, &[(0..3).into(), (1..2).into()]);
    assert_match(b"((.)%w*)", b"a1 b2", 0..2, &[(0..2).into(), (0..1).into()]);
}

#[test]
fn test_balanced_engine() {
    assert_match(b"%b()", b"(inner)", 0..7, &[]);
    assert_match(b"%b<>", b"<<a>>", 0..5, &[]);
    assert_match(b"a %b() c", b"a (bal) c", 0..9, &[]);
    assert_match(b"%b()", b"()", 0..2, &[]);
    assert_no_match(b"%b()", b"(unbalanced");
    assert_match(b"%b()", b"x()y", 1..3, &[]);
}

#[test]
fn test_frontier_engine() {
    assert_match(b"%f[a]a", b" a", 1..2, &[]);
    assert_match(b"%f[a]a", b"ba", 1..2, &[]);

    assert_no_match(b"%f[^%w]word", b"_word");
    assert_no_match(b"%f[^%w]word", b"1word");
    assert_no_match(b"%f[%s]a", b" a");

    assert_match(b"%f[a]a", b"a", 0..1, &[]);
    assert_match(b"%f[^a]b", b"b", 0..1, &[]);
}

#[test]
fn test_backtracking_engine() {
    assert_no_match(b"a*b", b"aaac");
    assert_no_match(b"a+b", b"aaac");
    assert_no_match(b"(ab)+a", b"abab");
    assert_match(b"(ab)+?a", b"abab", 0..3, &[(0..2).into()]);
    assert_match(b"(a*)b", b"aaab", 0..4, &[(0..3).into()]);
    assert_match(b"(a+)b", b"aaab", 0..4, &[(0..3).into()]);
    assert_match(b"a[bc]+d", b"abbcd", 0..5, &[]);
}

#[test]
fn test_empty_engine() {
    assert_match(b"", b"", 0..0, &[]);
    assert_match(b"", b"abc", 0..0, &[]);
    assert_no_match(b"a", b"");
    assert_match(b"a*", b"", 0..0, &[]);
    assert_no_match(b"a+", b"");
    assert_match(b"a?", b"", 0..0, &[]);
    assert_match(b"()", b"", 0..0, &[Capture::Position(0)]);
}

#[test]
fn test_find_offset_engine() {
    let pattern = b"b";
    let text = b"abc";
    let mut parser = Parser::new(pattern).unwrap();
    let ast = parser.parse().unwrap();
    let result = find_first_match(&ast, text, 1).unwrap();
    assert_eq!(result, (1..2, vec![]));

    assert!(find_first_match(&ast, text, 2).is_none());
}

#[test]
fn test_real_world_email_validation_engine() {
    assert_match(
        b"^[%w%.%+%-]+@[%w%.%+%-]+%.%w+$",
        b"user@example.com",
        0..16,
        &[],
    );
    assert_match(
        b"^[%w%.%+%-]+@[%w%.%+%-]+%.%w+$",
        b"user.name+tag-123@example-site.co.uk",
        0..36,
        &[],
    );

    assert_no_match(b"^[%w%.%+%-]+@[%w%.%+%-]+%.%w+$", b"user@example");
    assert_no_match(b"^[%w%.%+%-]+@[%w%.%+%-]+%.%w+$", b"@example.com");
    assert_no_match(b"^[%w%.%+%-]+@[%w%.%+%-]+%.%w+$", b"user@.com");
}

#[test]
fn test_extracting_data_with_captures_engine() {
    let result = find(b"(%d%d?)/(%d%d?)/(%d%d%d%d)", b"Date: 25/12/2023")
        .unwrap()
        .unwrap();
    let MatchRanges {
        full_match: full,
        captures,
    } = result;
    assert_eq!(full, 6..16);
    assert_eq!(captures[0], (6..8));
    assert_eq!(captures[1], (9..11));
    assert_eq!(captures[2], (12..16));

    let result = find(
        b"([%w%.%+%-]+)@([%w%.%+%-]+%.%w+)",
        b"Contact: john.doe@example.com",
    )
    .unwrap()
    .unwrap();
    let MatchRanges {
        full_match: full,
        captures,
    } = result;
    assert_eq!(full, 9..29);
    assert_eq!(captures[0], (9..17));
    assert_eq!(captures[1], (18..29));
}

#[test]
fn test_balanced_delimiters_engine() {
    assert_match(b"%b<>", b"<div><p>text</p></div>", 0..5, &[]);
    assert_match(b"%b()", b"(a + (b * c))", 0..13, &[]);
    assert_match(b"'%b\"\"'", b"'\"nested\"'", 0..10, &[]);
    assert_match(b"before %b() after", b"before (balanced) after", 0..23, &[]);
}

#[test]
fn test_frontier_patterns_engine() {
    assert_match(b"%f[%a]t%w+", b"start the test", 6..9, &[]);
    assert_match(b"%w+t%f[^%w]", b"start the test", 0..5, &[]);
    assert_match(b"%f[%w]word%f[^%w]", b"a word here", 2..6, &[]);
    assert_no_match(b"%f[%w]word%f[^%w]", b"aword here");
}

#[test]
fn test_complex_pattern_combinations_engine() {
    let pattern = b"<a%s+href=\"([^\"]+)\"[^>]*>([^<]*)</a>";
    let text = b"<p>Visit <a href=\"https://example.com\" class=\"link\">Example Site</a> for more info.</p>";

    let result = find(pattern, text).unwrap().unwrap();
    let MatchRanges {
        full_match: full,
        captures,
    } = result;
    assert_eq!(full, 9..68);
    assert_eq!(captures[0], (18..37));
    assert_eq!(captures[1], (52..64));

    assert_match(
        b"%f[%w][%u][%l]+%f[^%w]",
        b"This is a Test string",
        0..4,
        &[],
    );

    let result = find(b"([^,]+),([^,]+),([^,]+)", b"apple,orange,banana")
        .unwrap()
        .unwrap();
    let MatchRanges { captures, .. } = result;
    assert_eq!(captures[0], (0..5));
    assert_eq!(captures[1], (6..12));
    assert_eq!(captures[2], (13..19));
}

#[test]
fn test_optimization_cases_engine() {
    let mut parser = Parser::new(b"^abc").unwrap();
    let ast = parser.parse().unwrap();

    assert!(find_first_match(&ast, b"abcdef", 0).is_some());
    assert!(find_first_match(&ast, b"abcdef", 1).is_none());

    let mut parser = Parser::new(b"abc$").unwrap();
    let ast = parser.parse().unwrap();

    assert!(find_first_match(&ast, b"xyzabc", 0).is_some());
    assert!(find_first_match(&ast, b"abcxyz", 0).is_none());
}

#[test]
fn test_pattern_with_utf8_content_engine() {
    assert_match(b".", "привет".as_bytes(), 0..1, &[]);
    assert_match(b"..", "привет".as_bytes(), 0..2, &[]);

    assert_match(b"[%w]+", "привет123".as_bytes(), 12..15, &[]);

    assert_match(b"%a+", "hello привет".as_bytes(), 0..5, &[]);
}

#[test]
fn test_quantifiers_with_capturing_groups_engine() {
    assert_match(b"(a)+", b"aaa", 0..3, &[(2..3).into()]);
    assert_match(b"(ab)+", b"ababab", 0..6, &[(4..6).into()]);
    assert_match(b"(a)*", b"aaa", 0..3, &[(2..3).into()]);
    assert_match(b"(a)*", b"", 0..0, &[(0..0).into()]);
    assert_match(b"(a)?", b"a", 0..1, &[(0..1).into()]);
    assert_match(b"(a)?", b"", 0..0, &[(0..0).into()]);
    assert_match(b"(a)-", b"aaa", 0..0, &[(0..0).into()]);
}

#[test]
fn test_edge_cases_and_backtracking_engine() {
    assert_match(b"(a+)+", b"aaa", 0..3, &[(0..3).into()]);
    assert_match(b"[ab][cd]", b"ac", 0..2, &[]);
    assert_match(b"[ab][cd]", b"bd", 0..2, &[]);
    assert_no_match(b"[ab][cd]", b"ab");
    assert_match(b"a.-b", b"axxxbyyybzzz", 0..5, &[]);
    assert_match(b"a.*b", b"axxxbyyybzzz", 0..9, &[]);
    assert_match(
        b"(a*)(b?)b+",
        b"aaabbb",
        0..6,
        &[(0..3).into(), (3..4).into()],
    );
}

#[test]
fn test_real_world_patterns_advanced_engine() {
    let html = b"<div class='item'><span>Product: </span>Laptop</div><div class='price'>$999</div>";
    let pattern = b"<div class='([^']+)'>([^<]*<span>[^<]*</span>)?([^<]*)</div>";

    let result = find(pattern, html).unwrap().unwrap();
    let MatchRanges {
        full_match: full,
        captures,
    } = result;
    assert_eq!(full, 0..52);
    assert_eq!(captures[0], (12..16));
    assert_eq!(captures[1], (18..40));
    assert_eq!(captures[2], (40..46));

    let log_line = b"2023-04-15 14:23:45 ERROR [app.service] Failed to connect: timeout";
    let pattern = b"(%d+)%-(%d+)%-(%d+) (%d+):(%d+):(%d+) (%u+)";

    let result = find(pattern, log_line).unwrap().unwrap();
    let MatchRanges {
        full_match: full,
        captures,
    } = result;
    assert_eq!(full, 0..25);
    assert_eq!(captures[0], (0..4));
    assert_eq!(captures[1], (5..7));
    assert_eq!(captures[2], (8..10));
    assert_eq!(captures[3], (11..13));
    assert_eq!(captures[4], (14..16));
    assert_eq!(captures[5], (17..19));
    assert_eq!(captures[6], (20..25));
}

#[test]
fn test_subsequent_captures_engine() {
    assert_match(
        b"(%d%d%d%d)%-(%d%d)%-(%d%d)",
        b"2023-04-15",
        0..10,
        &[(0..4).into(), (5..7).into(), (8..10).into()],
    );

    assert_match(
        b"(%d+)_(%w+)_(%d+)",
        b"123_test_456",
        0..12,
        &[(0..3).into(), (4..8).into(), (9..12).into()],
    );
}
