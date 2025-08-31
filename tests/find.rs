use lsonar::{Error, find};

fn svec(items: &[&str]) -> Vec<String> {
    items.iter().map(|&s| s.to_string()).collect()
}

#[test]
fn test_negative_byte_classes() {
    assert_eq!(
        find(b"a b\tc", b"%S", None, false),
        Ok(Some((1, 1, vec![])))
    );
    assert_eq!(
        find(b"a b\tc", b"%S+", None, false),
        Ok(Some((1, 1, vec![])))
    );
    assert_eq!(find(b" b\tc", b"%S", None, false), Ok(Some((2, 2, vec![]))));
    assert_eq!(
        find(b"123abc", b"%D", None, false),
        Ok(Some((4, 4, vec![])))
    );
    assert_eq!(
        find(b"123abc", b"%D+", None, false),
        Ok(Some((4, 6, vec![])))
    );
    assert_eq!(
        find(b"abc_123", b"%W", None, false),
        Ok(Some((4, 4, vec![])))
    );
    assert_eq!(find(b"-abc-", b"%W", None, false), Ok(Some((1, 1, vec![]))));
    assert_eq!(
        find(b"abc123", b"%A", None, false),
        Ok(Some((4, 4, vec![])))
    );
    assert_eq!(
        find(b"abc123", b"%A+", None, false),
        Ok(Some((4, 6, vec![])))
    );
    assert_eq!(
        find("你a".as_bytes(), b"%A", None, false),
        Ok(Some((1, 1, vec![])))
    );
    assert_eq!(
        find("a你b".as_bytes(), b"%W", None, false),
        Ok(Some((2, 2, vec![])))
    );
}

#[test]
fn test_balanced_patterns() {
    assert_eq!(
        find(b"a(b(c)d)e", b"%b()", None, false),
        Ok(Some((2, 8, vec![])))
    );

    assert_eq!(
        find(b"a{b{c}d}e", b"%b{}", None, false),
        Ok(Some((2, 8, vec![])))
    );

    assert_eq!(
        find(b"a<b<c>d>e", b"%b<>", None, false),
        Ok(Some((2, 8, vec![])))
    );

    assert_eq!(
        find(b"a(b(c(d)e)f)g", b"%b()", None, false),
        Ok(Some((2, 12, vec![])))
    );

    assert_eq!(
        find(b"a(b(c)d)e", b"(%b())", None, false),
        Ok(Some((2, 8, vec![b"(b(c)d)".to_vec()])))
    );
}

#[test]
fn test_find_invalid_pattern() {
    assert!(matches!(
        find(b"abc", b"[", None, false),
        Err(Error::Parser(_))
    ));
    assert!(matches!(
        find(b"abc", b"(", None, false),
        Err(Error::Parser(_))
    ));
    assert!(matches!(
        find(b"abc", b"*", None, false),
        Err(Error::Parser(_))
    ));
    assert!(matches!(
        find(b"abc", b"%", None, false),
        Err(Error::Lexer(_))
    ));
    assert!(matches!(
        find(b"abc", b"%z", None, false),
        Err(Error::Parser(_)) | Err(Error::Lexer(_))
    ));
}

#[test]
fn test_plain_find() {
    assert_eq!(
        find(b"hello world", b"", None, true),
        Ok(Some((1, 0, vec![])))
    );
    assert_eq!(
        find(b"hello world", b"world", None, true),
        Ok(Some((7, 11, vec![])))
    );
    assert_eq!(
        find(b"hello world", b"hello", None, true),
        Ok(Some((1, 5, vec![])))
    );
    assert_eq!(find(b"hello world", b"not found", None, true), Ok(None));
    assert_eq!(
        find(b"hello world", b"", None, true),
        Ok(Some((1, 0, vec![])))
    );
}

#[test]
fn test_find_with_init() {
    assert_eq!(
        find(b"hello world", b"world", Some(6), false),
        Ok(Some((7, 11, vec![])))
    );
    assert_eq!(
        find(b"hello world", b"world", Some(7), false),
        Ok(Some((7, 11, vec![])))
    );
    assert_eq!(find(b"hello world", b"world", Some(8), false), Ok(None));
    assert_eq!(
        find(b"hello world", b"hello", Some(-11), false),
        Ok(Some((1, 5, vec![])))
    );
    assert_eq!(find(b"hello world", b"hello", Some(-5), false), Ok(None));
}

#[test]
fn test_find_pattern_with_captures() {
    assert_eq!(
        find(b"hello 123 world", b"(%d+)", None, false),
        Ok(Some((7, 9, vec![b"123".to_vec()])))
    );
    assert_eq!(
        find(b"name=John age=25", b"(%w+)=(%w+)", None, false),
        Ok(Some((1, 9, vec![b"name".to_vec(), b"John".to_vec()])))
    );
    assert_eq!(
        find(b"2023-04-15", b"(%d%d%d%d)%-(%d%d)%-(%d%d)", None, false),
        Ok(Some((
            1,
            10,
            vec![b"2023".to_vec(), b"04".to_vec(), b"15".to_vec()]
        )))
    );
}

#[test]
fn test_find_edge_cases() {
    assert_eq!(find(b"", b"", None, false), Ok(Some((1, 0, vec![]))));
    assert_eq!(find(b"hello", b"", None, false), Ok(Some((1, 0, vec![]))));
    assert_eq!(find(b"hello", b"^", None, false), Ok(Some((1, 0, vec![]))));
    assert_eq!(find(b"hello", b"$", None, false), Ok(Some((6, 5, vec![]))));
}
