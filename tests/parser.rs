use lsonar::{
    Error, LUA_MAXCAPTURES,
    ast::{AstNode, AstRoot, Quantifier, parse_pattern},
    charset::CharSet,
    lexer::Token,
};

fn parse_ok(pattern: &[u8]) -> AstRoot {
    parse_pattern(pattern).unwrap_or_else(|_| {
        panic!(
            "Parser failed for pattern: {}",
            str::from_utf8(pattern).unwrap()
        )
    })
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
        parse_ok(b"abc"),
        &[
            AstNode::Literal(b'a'),
            AstNode::Literal(b'b'),
            AstNode::Literal(b'c')
        ]
    );
    assert_eq!(
        parse_ok(b"a.c"),
        &[AstNode::Literal(b'a'), AstNode::Any, AstNode::Literal(b'c')]
    );
    assert_eq!(
        parse_ok(b"a%dc"),
        &[
            AstNode::Literal(b'a'),
            AstNode::Class(b'd', false),
            AstNode::Literal(b'c')
        ]
    );
    assert_eq!(
        parse_ok(b"a%Dc"),
        &[
            AstNode::Literal(b'a'),
            AstNode::Class(b'd', true),
            AstNode::Literal(b'c')
        ]
    );
}

#[test]
fn test_anchors_parser() {
    assert_eq!(
        parse_ok(b"^abc$"),
        &[
            AstNode::AnchorStart,
            AstNode::Literal(b'a'),
            AstNode::Literal(b'b'),
            AstNode::Literal(b'c'),
            AstNode::AnchorEnd
        ]
    );
    assert_eq!(
        parse_ok(b"abc$"),
        &[
            AstNode::Literal(b'a'),
            AstNode::Literal(b'b'),
            AstNode::Literal(b'c'),
            AstNode::AnchorEnd
        ]
    );
    assert_eq!(
        parse_ok(b"^abc"),
        &[
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
        parse_ok(b"a*"),
        &[quantified(AstNode::Literal(b'a'), Quantifier::Star)]
    );
    assert_eq!(
        parse_ok(b"a+"),
        &[quantified(AstNode::Literal(b'a'), Quantifier::Plus)]
    );
    assert_eq!(
        parse_ok(b"a?"),
        &[quantified(AstNode::Literal(b'a'), Quantifier::Question)]
    );
    assert_eq!(
        parse_ok(b"a-"),
        &[quantified(AstNode::Literal(b'a'), Quantifier::Minus)]
    );
    assert_eq!(
        parse_ok(b"a.*c+d?e-"),
        &[
            AstNode::Literal(b'a'),
            quantified(AstNode::Any, Quantifier::Star),
            quantified(AstNode::Literal(b'c'), Quantifier::Plus),
            quantified(AstNode::Literal(b'd'), Quantifier::Question),
            quantified(AstNode::Literal(b'e'), Quantifier::Minus),
        ]
    );
    assert_eq!(
        parse_ok(b"%d+"),
        &[quantified(AstNode::Class(b'd', false), Quantifier::Plus)]
    );
    assert_eq!(
        parse_ok(b".*"),
        &[quantified(AstNode::Any, Quantifier::Star)]
    );
}

#[test]
fn test_sets_parser() {
    assert_eq!(
        parse_ok(b"[]"),
        &[AstNode::Set(make_set(&[], &[], &[], false))]
    );
    assert_eq!(
        parse_ok(b"[abc]"),
        &[AstNode::Set(make_set(b"abc", &[], &[], false))]
    );
    assert_eq!(
        parse_ok(b"[^abc]"),
        &[AstNode::Set(make_set(b"abc", &[], &[], true))]
    );
    assert_eq!(
        parse_ok(b"[a-c]"),
        &[AstNode::Set(make_set(&[], &[(b'a', b'c')], &[], false))]
    );
    assert_eq!(
        parse_ok(b"[^a-c]"),
        &[AstNode::Set(make_set(&[], &[(b'a', b'c')], &[], true))]
    );
    assert_eq!(
        parse_ok(b"[a.^$]"),
        &[AstNode::Set(make_set(b"a.^$", &[], &[], false))]
    );
    assert_eq!(
        parse_ok(b"[%a]"),
        &[AstNode::Set(make_set(&[], &[], b"a", false))]
    );
    assert_eq!(
        parse_ok(b"[%%]"),
        &[AstNode::Set(make_set(b"%", &[], &[], false))]
    );
    assert_eq!(
        parse_ok(b"[-abc]"),
        &[AstNode::Set(make_set(b"-abc", &[], &[], false))]
    );
    assert_eq!(
        parse_ok(b"[abc-]"),
        &[AstNode::Set(make_set(b"abc-", &[], &[], false))]
    );
}

#[test]
fn test_set_quantifier_parser() {
    assert_eq!(
        parse_ok(b"[abc]*"),
        &[quantified(
            AstNode::Set(make_set(b"abc", &[], &[], false)),
            Quantifier::Star
        )]
    );
}

#[test]
fn test_captures_parser() {
    assert_eq!(
        parse_ok(b"()"),
        &[AstNode::Capture {
            index: 1,
            inner: vec![]
        }]
    );
    assert_eq!(
        parse_ok(b"(a)"),
        &[AstNode::Capture {
            index: 1,
            inner: vec![AstNode::Literal(b'a')]
        }]
    );
    assert_eq!(
        parse_ok(b"(a%d+)"),
        &[AstNode::Capture {
            index: 1,
            inner: vec![
                AstNode::Literal(b'a'),
                quantified(AstNode::Class(b'd', false), Quantifier::Plus)
            ]
        }]
    );
    assert_eq!(
        parse_ok(b"(a(b)c)"),
        &[AstNode::Capture {
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
        parse_ok(b"(a)?"),
        &[quantified(
            AstNode::Capture {
                index: 1,
                inner: vec![AstNode::Literal(b'a')]
            },
            Quantifier::Question
        )]
    );
    assert_eq!(
        parse_ok(b"a?b"),
        &[
            quantified(AstNode::Literal(b'a'), Quantifier::Question),
            AstNode::Literal(b'b')
        ]
    );
    assert_eq!(
        parse_ok(b"a-b"),
        &[
            quantified(AstNode::Literal(b'a'), Quantifier::Minus),
            AstNode::Literal(b'b')
        ]
    );
}

#[test]
fn test_balanced_frontier_parser() {
    assert_eq!(parse_ok(b"%b()"), &[AstNode::Balanced(b'(', b')')]);
    assert_eq!(
        parse_ok(b"%f[ac]"),
        &[AstNode::Frontier(make_set(b"ac", &[], &[], false))]
    );
}

#[test]
fn test_complex_parser() {
    assert_eq!(
        parse_ok(b"^(%b())%d*$"),
        &[
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
fn test_escaped_rparen_rbracket_without_panic() {
    assert_eq!(parse_ok(b"%]"), &[AstNode::Literal(b']')]);
    assert_eq!(parse_ok(b"%)"), &[AstNode::Literal(b')')])
}

#[test]
fn test_throw_parser_errors() {
    assert!(matches!(
        parse_pattern(b"xxxx("),
        Err(Error::ExpectedToken {
            pos: 5,
            expected: Token::RParen,
            actual: None
        })
    ));
    assert!(matches!(
        parse_pattern(b"xx)"),
        Err(Error::UnexpectedToken { lit: b')', pos: 2 })
    ));
    assert!(matches!(
        parse_pattern(b"x]"),
        Err(Error::UnexpectedToken { lit: b']', pos: 1 })
    ));
    assert!(matches!(
        parse_pattern(b"["),
        Err(Error::ExpectedToken {
            pos: 1,
            expected: Token::RBracket,
            actual: None
        })
    ));
    assert!(matches!(
        parse_pattern(b"*"),
        Err(Error::UnexpectedToken { lit: b'*', pos: 0 })
    ));
    assert!(matches!(
        parse_pattern(b"^*"),
        Err(Error::UnexpectedToken { pos: 1, lit: b'*' })
    ));
    assert!(matches!(
        parse_pattern(b"$+"),
        Err(Error::UnexpectedToken { pos: 1, lit: b'+' })
    ));
    assert!(matches!(
        parse_pattern(b"%b"),
        Err(Error::MissingArgs { pos: 2 })
    ));
    assert!(matches!(
        parse_pattern(b"%bx"),
        Err(Error::MissingArgs { pos: 3 })
    ));
    assert!(matches!(
        parse_pattern(b"%f"),
        Err(Error::ExpectedToken {
            pos: 2,
            expected: Token::LBracket,
            actual: None
        })
    ));
    assert!(matches!(
        parse_pattern(b"%fa"),
        Err(Error::ExpectedToken {
            pos: 2,
            expected: Token::LBracket,
            actual: Some(Token::Literal(b'a'))
        })
    ));
    assert!(matches!(
        parse_pattern(b"%f["),
        Err(Error::ExpectedToken {
            pos: 3,
            expected: Token::RBracket,
            actual: None
        })
    ));
    assert!(matches!(
        parse_pattern(b"%f[a"),
        Err(Error::ExpectedToken {
            pos: 4,
            expected: Token::RBracket,
            actual: None
        })
    ));
    assert!(matches!(
        parse_pattern(b"%z"),
        Err(Error::UnknownClass { pos: 0, lit: b'z' })
    ));

    assert_eq!(parse_ok(b"%1"), &[AstNode::CaptureRef(1)]);

    let too_many_captures = "()".repeat(LUA_MAXCAPTURES + 1);
    assert!(matches!(
        parse_pattern(too_many_captures.as_bytes()),
        Err(Error::Captures(c)) if c == LUA_MAXCAPTURES + 1
    ));
}

#[test]
fn test_special_byte_edge_cases_parser() {
    assert_eq!(
        parse_ok(b"[%%]"),
        &[AstNode::Set(make_set(b"%", &[], &[], false))]
    );
    assert_eq!(
        parse_ok(b"[%-]"),
        &[AstNode::Set(make_set(b"-", &[], &[], false))]
    );
    assert_eq!(
        parse_ok(b"[%]]"),
        &[AstNode::Set(make_set(b"]", &[], &[], false))]
    );
    assert_eq!(
        parse_ok(b"[%[]"),
        &[AstNode::Set(make_set(b"[", &[], &[], false))]
    );

    assert_eq!(
        parse_ok(b"%*+%?"),
        &[
            quantified(AstNode::Literal(b'*'), Quantifier::Plus),
            AstNode::Literal(b'?')
        ]
    );
    assert_eq!(
        parse_ok(b"[%[]"),
        &[AstNode::Set(make_set(b"[", &[], &[], false))]
    );
}

#[test]
fn test_nested_complex_patterns_parser() {
    assert_eq!(
        parse_ok(b"((a+)?(b*))+"),
        &[quantified(
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
        parse_ok(b"(%f[%a]%w+)"),
        &[AstNode::Capture {
            index: 1,
            inner: vec![
                AstNode::Frontier(make_set(&[], &[], b"a", false)),
                quantified(AstNode::Class(b'w', false), Quantifier::Plus)
            ]
        }]
    );
}

#[test]
fn test_real_world_patterns_parser() {
    assert!(!parse_ok(b"https?://[%w%.%-%+]+%.%w+").is_empty());

    assert!(!parse_ok(b"^[%w%.%+%-]+@[%w%.%+%-]+%.%w+$").is_empty());

    assert!(!parse_ok(b"(%d%d?)/(%d%d?)/(%d%d%d%d)").is_empty());

    assert!(!parse_ok(b"(%d+)%.(%d+)%.(%d+)%.(%d+)").is_empty());

    assert!(!parse_ok(b"\"([^\"]+)\":%s*\"([^\"]*)\"").is_empty());
}

#[test]
fn test_special_lua_pattern_features_parser() {
    assert!(!parse_ok(b"%1").is_empty());
    assert!(!parse_ok(b"(.)%1").is_empty());
    assert!(!parse_ok(b"%b{}").is_empty());
    assert!(!parse_ok(b"%f[%a]").is_empty());
}
