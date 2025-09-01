use lsonar::{Error, find, lexer::Token};

#[test]
fn test_negative_byte_classes() {
    assert_eq!(
        find(b"a b\tc", b"%S", None, false),
        Ok(Some((1, 1, vec![]).into()))
    );
    assert_eq!(
        find(b"a b\tc", b"%S+", None, false),
        Ok(Some((1, 1, vec![]).into()))
    );
    assert_eq!(
        find(b" b\tc", b"%S", None, false),
        Ok(Some((2, 2, vec![]).into()))
    );
    assert_eq!(
        find(b"123abc", b"%D", None, false),
        Ok(Some((4, 4, vec![]).into()))
    );
    assert_eq!(
        find(b"123abc", b"%D+", None, false),
        Ok(Some((4, 6, vec![]).into()))
    );
    assert_eq!(
        find(b"abc_123", b"%W", None, false),
        Ok(Some((4, 4, vec![]).into()))
    );
    assert_eq!(
        find(b"-abc-", b"%W", None, false),
        Ok(Some((1, 1, vec![]).into()))
    );
    assert_eq!(
        find(b"abc123", b"%A", None, false),
        Ok(Some((4, 4, vec![]).into()))
    );
    assert_eq!(
        find(b"abc123", b"%A+", None, false),
        Ok(Some((4, 6, vec![]).into()))
    );
    assert_eq!(
        find("你a".as_bytes(), b"%A", None, false),
        Ok(Some((1, 1, vec![]).into()))
    );
    assert_eq!(
        find("a你b".as_bytes(), b"%W", None, false),
        Ok(Some((2, 2, vec![]).into()))
    );
}

#[test]
fn test_balanced_patterns() {
    assert_eq!(
        find(b"a(b(c)d)e", b"%b()", None, false),
        Ok(Some((2, 8, vec![]).into()))
    );

    assert_eq!(
        find(b"a{b{c}d}e", b"%b{}", None, false),
        Ok(Some((2, 8, vec![]).into()))
    );

    assert_eq!(
        find(b"a<b<c>d>e", b"%b<>", None, false),
        Ok(Some((2, 8, vec![]).into()))
    );

    assert_eq!(
        find(b"a(b(c(d)e)f)g", b"%b()", None, false),
        Ok(Some((2, 12, vec![]).into()))
    );

    assert_eq!(
        find(b"a(b(c)d)e", b"(%b())", None, false),
        Ok(Some((2, 8, vec![b"(b(c)d)".into()]).into()))
    );
}

#[test]
fn test_find_invalid_pattern() {
    assert!(matches!(
        find(b"abc", b"[", None, false),
        Err(Error::ExpectedToken {
            pos: 1,
            expected: Token::RBracket,
            actual: None
        })
    ));
    assert!(matches!(
        find(b"abc", b"(", None, false),
        Err(Error::ExpectedToken {
            pos: 1,
            expected: Token::RParen,
            actual: None
        })
    ));
    assert!(matches!(
        find(b"abc", b"*", None, false),
        Err(Error::UnexpectedToken { pos: 0, lit: b'*' })
    ));
    assert!(matches!(
        find(b"abc", b"%", None, false),
        Err(Error::UnexpectedEnd { pos: 1 })
    ));
    assert!(matches!(
        find(b"abc", b"%z", None, false),
        Err(Error::UnknownClass { pos: 0, lit: b'z' })
    ));
}

#[test]
fn test_plain_find() {
    assert_eq!(
        find(b"hello world", b"", None, true),
        Ok(Some((1, 0, vec![]).into()))
    );
    assert_eq!(
        find(b"hello world", b"world", None, true),
        Ok(Some((7, 11, vec![]).into()))
    );
    assert_eq!(
        find(b"hello world", b"hello", None, true),
        Ok(Some((1, 5, vec![]).into()))
    );
    assert_eq!(find(b"hello world", b"not found", None, true), Ok(None));
    assert_eq!(
        find(b"hello world", b"", None, true),
        Ok(Some((1, 0, vec![]).into()))
    );
}

#[test]
fn test_find_with_init() {
    assert_eq!(
        find(b"hello world", b"world", Some(6), false),
        Ok(Some((7, 11, vec![]).into()))
    );
    assert_eq!(
        find(b"hello world", b"world", Some(7), false),
        Ok(Some((7, 11, vec![]).into()))
    );
    assert_eq!(find(b"hello world", b"world", Some(8), false), Ok(None));
    assert_eq!(
        find(b"hello world", b"hello", Some(-11), false),
        Ok(Some((1, 5, vec![]).into()))
    );
    assert_eq!(find(b"hello world", b"hello", Some(-5), false), Ok(None));
}

#[test]
fn test_find_pattern_with_captures() {
    assert_eq!(
        find(b"hello 123 world", b"(%d+)", None, false),
        Ok(Some((7, 9, vec![b"123".into()]).into()))
    );
    assert_eq!(
        find(b"name=John age=25", b"(%w+)=(%w+)", None, false),
        Ok(Some((1, 9, vec![b"name".into(), b"John".into()]).into()))
    );
    assert_eq!(
        find(b"2023-04-15", b"(%d%d%d%d)%-(%d%d)%-(%d%d)", None, false),
        Ok(Some(
            (1, 10, vec![b"2023".into(), b"04".into(), b"15".into()]).into()
        ))
    );
}

#[test]
fn test_find_edge_cases() {
    assert_eq!(find(b"", b"", None, false), Ok(Some((1, 0, vec![]).into())));
    assert_eq!(
        find(b"hello", b"", None, false),
        Ok(Some((1, 0, vec![]).into()))
    );
    assert_eq!(
        find(b"hello", b"^", None, false),
        Ok(Some((1, 0, vec![]).into()))
    );
    assert_eq!(
        find(b"hello", b"$", None, false),
        Ok(Some((6, 5, vec![]).into()))
    );
}

#[test]
fn test_find_positions() {
    assert_eq!(
        find(b"abcdef", b"abc()def()", None, false),
        Ok(Some(
            (1, 6, vec![b"4".to_vec().into(), b"7".to_vec().into()]).into()
        ))
    );
}
