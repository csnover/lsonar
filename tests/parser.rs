use lsonar::charset::CharSet;
use lsonar::{AstNode, Error, LUA_MAXCAPTURES, Parser, Quantifier, Result};

fn parse_ok(pattern: &str) -> Vec<AstNode> {
    Parser::new(pattern)
        .expect("Parser::new failed")
        .parse()
        .expect(&format!("Parser failed for pattern: {}", pattern))
}

fn parse_err(pattern: &str) -> Result<Vec<AstNode>> {
    let mut parser = Parser::new(pattern)?;
    parser.parse()
}

fn quantified(item: AstNode, quantifier: Quantifier) -> AstNode {
    AstNode::Quantified {
        item: Box::new(item),
        quantifier,
    }
}

fn make_set(bytes: &[u8], ranges: &[(u8, u8)], classes: &[u8], negated: bool) -> CharSet {
    let mut set = CharSet::new();
    for &b in bytes {
        set.add_byte(b);
    }
    for &(s, e) in ranges {
        set.add_range(s, e).unwrap();
    }
    for &c in classes {
        set.add_class(c).unwrap();
    }
    if negated {
        set.invert();
    }
    set
}

#[test]
fn test_simple_sequence_parser() {
    assert_eq!(
        parse_ok("abc"),
        vec![
            AstNode::Literal(b'a'),
            AstNode::Literal(b'b'),
            AstNode::Literal(b'c')
        ]
    );
    assert_eq!(
        parse_ok("a.c"),
        vec![AstNode::Literal(b'a'), AstNode::Any, AstNode::Literal(b'c')]
    );
    assert_eq!(
        parse_ok("a%dc"),
        vec![
            AstNode::Literal(b'a'),
            AstNode::Class(b'd', false),
            AstNode::Literal(b'c')
        ]
    );
    assert_eq!(
        parse_ok("a%Dc"),
        vec![
            AstNode::Literal(b'a'),
            AstNode::Class(b'd', true),
            AstNode::Literal(b'c')
        ]
    );
}

#[test]
fn test_anchors_parser() {
    assert_eq!(
        parse_ok("^abc$"),
        vec![
            AstNode::AnchorStart,
            AstNode::Literal(b'a'),
            AstNode::Literal(b'b'),
            AstNode::Literal(b'c'),
            AstNode::AnchorEnd
        ]
    );
    assert_eq!(
        parse_ok("abc$"),
        vec![
            AstNode::Literal(b'a'),
            AstNode::Literal(b'b'),
            AstNode::Literal(b'c'),
            AstNode::AnchorEnd
        ]
    );
    assert_eq!(
        parse_ok("^abc"),
        vec![
            AstNode::AnchorStart,
            AstNode::Literal(b'a'),
            AstNode::Literal(b'b'),
            AstNode::Literal(b'c')
        ]
    );
}

#[test]
fn test_quantifiers_parser() {
    assert_eq!(
        parse_ok("a*"),
        vec![quantified(AstNode::Literal(b'a'), Quantifier::Star)]
    );
    assert_eq!(
        parse_ok("a+"),
        vec![quantified(AstNode::Literal(b'a'), Quantifier::Plus)]
    );
    assert_eq!(
        parse_ok("a?"),
        vec![quantified(AstNode::Literal(b'a'), Quantifier::Question)]
    );
    assert_eq!(
        parse_ok("a-"),
        vec![quantified(AstNode::Literal(b'a'), Quantifier::Minus)]
    );
    assert_eq!(
        parse_ok("a.*c+d?e-"),
        vec![
            AstNode::Literal(b'a'),
            quantified(AstNode::Any, Quantifier::Star),
            quantified(AstNode::Literal(b'c'), Quantifier::Plus),
            quantified(AstNode::Literal(b'd'), Quantifier::Question),
            quantified(AstNode::Literal(b'e'), Quantifier::Minus),
        ]
    );
    assert_eq!(
        parse_ok("%d+"),
        vec![quantified(AstNode::Class(b'd', false), Quantifier::Plus)]
    );
    assert_eq!(
        parse_ok(".*"),
        vec![quantified(AstNode::Any, Quantifier::Star)]
    );
}

#[test]
fn test_sets_parser() {
    assert_eq!(
        parse_ok("[]"),
        vec![AstNode::Set(make_set(&[], &[], &[], false))]
    );
    assert_eq!(
        parse_ok("[abc]"),
        vec![AstNode::Set(make_set(&[b'a', b'b', b'c'], &[], &[], false))]
    );
    assert_eq!(
        parse_ok("[^abc]"),
        vec![AstNode::Set(make_set(&[b'a', b'b', b'c'], &[], &[], true))]
    );
    assert_eq!(
        parse_ok("[a-c]"),
        vec![AstNode::Set(make_set(&[], &[(b'a', b'c')], &[], false))]
    );
    assert_eq!(
        parse_ok("[^a-c]"),
        vec![AstNode::Set(make_set(&[], &[(b'a', b'c')], &[], true))]
    );
    assert_eq!(
        parse_ok("[a-c%d]"),
        vec![AstNode::Set(make_set(&[], &[(b'a', b'c')], &[b'd'], false))]
    );
    assert_eq!(
        parse_ok("[a.^$]"),
        vec![AstNode::Set(make_set(
            &[b'a', b'.', b'^', b'$'],
            &[],
            &[],
            false
        ))]
    );
    assert_eq!(
        parse_ok("[%a]"),
        vec![AstNode::Set(make_set(&[], &[], &[b'a'], false))]
    );
    assert_eq!(
        parse_ok("[%]"),
        vec![AstNode::Set(make_set(&[b'%'], &[], &[], false))]
    );
    assert_eq!(
        parse_ok("[-abc]"),
        vec![AstNode::Set(make_set(
            &[b'-', b'a', b'b', b'c'],
            &[],
            &[],
            false
        ))]
    );
    assert_eq!(
        parse_ok("[abc-]"),
        vec![AstNode::Set(make_set(
            &[b'a', b'b', b'c', b'-'],
            &[],
            &[],
            false
        ))]
    );
}

#[test]
fn test_set_quantifier_parser() {
    assert_eq!(
        parse_ok("[abc]*"),
        vec![quantified(
            AstNode::Set(make_set(&[b'a', b'b', b'c'], &[], &[], false)),
            Quantifier::Star
        )]
    );
}

#[test]
fn test_captures_parser() {
    assert_eq!(
        parse_ok("()"),
        vec![AstNode::Capture {
            index: 1,
            inner: vec![]
        }]
    );
    assert_eq!(
        parse_ok("(a)"),
        vec![AstNode::Capture {
            index: 1,
            inner: vec![AstNode::Literal(b'a')]
        }]
    );
    assert_eq!(
        parse_ok("(a%d+)"),
        vec![AstNode::Capture {
            index: 1,
            inner: vec![
                AstNode::Literal(b'a'),
                quantified(AstNode::Class(b'd', false), Quantifier::Plus)
            ]
        }]
    );
    assert_eq!(
        parse_ok("(a(b)c)"),
        vec![AstNode::Capture {
            index: 1,
            inner: vec![
                AstNode::Literal(b'a'),
                AstNode::Capture {
                    index: 2,
                    inner: vec![AstNode::Literal(b'b')]
                },
                AstNode::Literal(b'c')
            ]
        }]
    );
    assert_eq!(
        parse_ok("(a)?"),
        vec![quantified(
            AstNode::Capture {
                index: 1,
                inner: vec![AstNode::Literal(b'a')]
            },
            Quantifier::Question
        )]
    );
    assert_eq!(
        parse_ok("a?b"),
        vec![
            quantified(AstNode::Literal(b'a'), Quantifier::Question),
            AstNode::Literal(b'b')
        ]
    );
    assert_eq!(
        parse_ok("a-b"),
        vec![
            quantified(AstNode::Literal(b'a'), Quantifier::Minus),
            AstNode::Literal(b'b')
        ]
    );
}

#[test]
fn test_balanced_frontier_parser() {
    assert_eq!(parse_ok("%b()"), vec![AstNode::Balanced(b'(', b')')]);
    assert_eq!(
        parse_ok("%f[ac]"),
        vec![AstNode::Frontier(make_set(&[b'a', b'c'], &[], &[], false))]
    );
}

#[test]
fn test_complex_parser() {
    assert_eq!(
        parse_ok("^(%b())%d*$"),
        vec![
            AstNode::AnchorStart,
            AstNode::Capture {
                index: 1,
                inner: vec![AstNode::Balanced(b'(', b')')]
            },
            quantified(AstNode::Class(b'd', false), Quantifier::Star),
            AstNode::AnchorEnd
        ]
    );
}

#[test]
fn test_throw_parser_errors() {
    assert!(
        matches!(parse_err("("), Err(Error::Parser(s)) if s.contains("malformed pattern (unexpected end, expected RParen)"))
    );
    assert!(matches!(parse_err(")"), Err(Error::Parser(s)) if s.contains("unexpected ')'")));
    assert!(
        matches!(parse_err("["), Err(Error::Parser(s)) if s.contains("unfinished character class"))
    );
    assert_eq!(parse_ok("]"), vec![AstNode::Literal(b']')]);
    assert!(matches!(parse_err("*"), Err(Error::Parser(s)) if s.contains("must follow an item")));
    assert!(matches!(parse_err("^*"), Err(Error::Parser(s)) if s.contains("cannot be quantified")));
    assert!(matches!(parse_err("$+"), Err(Error::Parser(s)) if s.contains("cannot be quantified")));
    assert!(matches!(parse_err("%b"), Err(Error::Lexer(s)) if s.contains("needs two characters")));
    assert!(matches!(parse_err("%bx"), Err(Error::Lexer(s)) if s.contains("needs two characters")));
    assert!(matches!(parse_err("%f"), Err(Error::Parser(s)) if s.contains("missing '[' after %f")));
    assert!(
        matches!(parse_err("%fa"), Err(Error::Parser(s)) if s.contains("missing '[' after %f"))
    );
    assert!(
        matches!(parse_err("%f["), Err(Error::Parser(s)) if s.contains("unfinished character class"))
    );
    assert!(
        matches!(parse_err("%f[a"), Err(Error::Parser(s)) if s.contains("unfinished character class"))
    );
    assert!(matches!(parse_err("%z"), Err(Error::Lexer(_))));

    assert_eq!(parse_ok("%1"), vec![AstNode::CaptureRef(1)]);

    let too_many_captures = "()".repeat(LUA_MAXCAPTURES + 1);
    assert!(
        matches!(parse_err(&too_many_captures), Err(Error::Parser(s)) if s.contains("too many captures"))
    );
}

#[test]
fn test_special_byte_edge_cases_parser() {
    assert_eq!(
        parse_ok("[%%]"),
        vec![AstNode::Set(make_set(&[b'%'], &[], &[], false))]
    );
    assert_eq!(
        parse_ok("[%-]"),
        vec![AstNode::Set(make_set(&[b'%', b'-'], &[], &[], false))]
    );
    assert_eq!(
        parse_ok("[%]]"),
        vec![
            AstNode::Set(make_set(&[b'%'], &[], &[], false)),
            AstNode::Literal(b']')
        ]
    );
    assert_eq!(
        parse_ok("[%[]"),
        vec![AstNode::Set(make_set(&[b'%', b'['], &[], &[], false))]
    );

    assert_eq!(
        parse_ok("%*+%?"),
        vec![
            quantified(AstNode::Literal(b'*'), Quantifier::Plus),
            AstNode::Literal(b'?')
        ]
    );
    assert_eq!(
        parse_ok("[%[]"),
        vec![AstNode::Set(make_set(&[b'%', b'['], &[], &[], false))]
    );
}

#[test]
fn test_nested_complex_patterns_parser() {
    assert_eq!(
        parse_ok("((a+)?(b*))+"),
        vec![quantified(
            AstNode::Capture {
                index: 1,
                inner: vec![
                    quantified(
                        AstNode::Capture {
                            index: 2,
                            inner: vec![quantified(AstNode::Literal(b'a'), Quantifier::Plus)]
                        },
                        Quantifier::Question
                    ),
                    AstNode::Capture {
                        index: 3,
                        inner: vec![quantified(AstNode::Literal(b'b'), Quantifier::Star)]
                    }
                ]
            },
            Quantifier::Plus
        )]
    );

    assert_eq!(
        parse_ok("(%f[%a]%w+)"),
        vec![AstNode::Capture {
            index: 1,
            inner: vec![
                AstNode::Frontier(make_set(&[], &[], &[b'a'], false)),
                quantified(AstNode::Class(b'w', false), Quantifier::Plus)
            ]
        }]
    );
}

#[test]
fn test_real_world_patterns_parser() {
    assert!(parse_ok("https?://[%w%.%-%+]+%.%w+").len() > 0);

    assert!(parse_ok("^[%w%.%+%-]+@[%w%.%+%-]+%.%w+$").len() > 0);

    assert!(parse_ok("(%d%d?)/(%d%d?)/(%d%d%d%d)").len() > 0);

    assert!(parse_ok("(%d+)%.(%d+)%.(%d+)%.(%d+)").len() > 0);

    assert!(parse_ok("\"([^\"]+)\":%s*\"([^\"]*)\"").len() > 0);
}

#[test]
fn test_special_lua_pattern_features_parser() {
    assert!(parse_ok("%1").len() > 0);
    assert!(parse_ok("(.)%1").len() > 0);
    assert!(parse_ok("%b{}").len() > 0);
    assert!(parse_ok("%f[%a]").len() > 0);
}
