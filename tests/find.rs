use lsonar::{Error, find};

fn svec(items: &[&str]) -> Vec<String> {
    items.iter().map(|&s| s.to_string()).collect()
}

#[test]
fn test_negative_byte_classes() {
    assert_eq!(find("a b\tc", "%S", None, false), Ok(Some((1, 1, vec![]))));
    assert_eq!(find("a b\tc", "%S+", None, false), Ok(Some((1, 1, vec![]))));
    assert_eq!(find(" b\tc", "%S", None, false), Ok(Some((2, 2, vec![]))));
    assert_eq!(find("123abc", "%D", None, false), Ok(Some((4, 4, vec![]))));
    assert_eq!(find("123abc", "%D+", None, false), Ok(Some((4, 6, vec![]))));
    assert_eq!(find("abc_123", "%W", None, false), Ok(Some((4, 4, vec![]))));
    assert_eq!(find("-abc-", "%W", None, false), Ok(Some((1, 1, vec![]))));
    assert_eq!(find("abc123", "%A", None, false), Ok(Some((4, 4, vec![]))));
    assert_eq!(find("abc123", "%A+", None, false), Ok(Some((4, 6, vec![]))));
    assert_eq!(find("你a", "%A", None, false), Ok(Some((1, 1, vec![]))));
    assert_eq!(find("a你b", "%W", None, false), Ok(Some((2, 2, vec![]))));
}

#[test]
fn test_balanced_patterns() {
    assert_eq!(
        find("a(b(c)d)e", "%b()", None, false),
        Ok(Some((2, 8, vec![])))
    );

    assert_eq!(
        find("a{b{c}d}e", "%b{}", None, false),
        Ok(Some((2, 8, vec![])))
    );

    assert_eq!(
        find("a<b<c>d>e", "%b<>", None, false),
        Ok(Some((2, 8, vec![])))
    );

    assert_eq!(
        find("a(b(c(d)e)f)g", "%b()", None, false),
        Ok(Some((2, 12, vec![])))
    );

    assert_eq!(
        find("a(b(c)d)e", "(%b())", None, false),
        Ok(Some((2, 8, svec(&["(b(c)d)"]))))
    );
}

#[test]
fn test_find_invalid_pattern() {
    assert!(matches!(
        find("abc", "[", None, false),
        Err(Error::Parser(_))
    ));
    assert!(matches!(
        find("abc", "(", None, false),
        Err(Error::Parser(_))
    ));
    assert!(matches!(
        find("abc", "*", None, false),
        Err(Error::Parser(_))
    ));
    assert!(matches!(
        find("abc", "%", None, false),
        Err(Error::Lexer(_))
    ));
    assert!(matches!(
        find("abc", "%z", None, false),
        Err(Error::Parser(_)) | Err(Error::Lexer(_))
    ));
}

#[test]
fn test_plain_find() {
    assert_eq!(
        find("hello world", "", None, true),
        Ok(Some((1, 0, vec![])))
    );
    assert_eq!(
        find("hello world", "world", None, true),
        Ok(Some((7, 11, vec![])))
    );
    assert_eq!(
        find("hello world", "hello", None, true),
        Ok(Some((1, 5, vec![])))
    );
    assert_eq!(find("hello world", "not found", None, true), Ok(None));
    assert_eq!(
        find("hello world", "", None, true),
        Ok(Some((1, 0, vec![])))
    );
}

#[test]
fn test_find_with_init() {
    assert_eq!(
        find("hello world", "world", Some(6), false),
        Ok(Some((7, 11, vec![])))
    );
    assert_eq!(
        find("hello world", "world", Some(7), false),
        Ok(Some((7, 11, vec![])))
    );
    assert_eq!(find("hello world", "world", Some(8), false), Ok(None));
    assert_eq!(
        find("hello world", "hello", Some(-11), false),
        Ok(Some((1, 5, vec![])))
    );
    assert_eq!(find("hello world", "hello", Some(-5), false), Ok(None));
}

#[test]
fn test_find_pattern_with_captures() {
    assert_eq!(
        find("hello 123 world", "(%d+)", None, false),
        Ok(Some((7, 9, svec(&["123"]))))
    );
    assert_eq!(
        find("name=John age=25", "(%w+)=(%w+)", None, false),
        Ok(Some((1, 9, svec(&["name", "John"]))))
    );
    assert_eq!(
        find("2023-04-15", "(%d%d%d%d)%-(%d%d)%-(%d%d)", None, false),
        Ok(Some((1, 10, svec(&["2023", "04", "15"]))))
    );
}

#[test]
fn test_find_edge_cases() {
    assert_eq!(find("", "", None, false), Ok(Some((1, 0, vec![]))));
    assert_eq!(find("hello", "", None, false), Ok(Some((1, 0, vec![]))));
    assert_eq!(find("hello", "^", None, false), Ok(Some((1, 0, vec![]))));
    assert_eq!(find("hello", "$", None, false), Ok(Some((6, 5, vec![]))));
}
