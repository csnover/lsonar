use lsonar::r#match;

#[test]
fn test_simple_match() {
    assert_eq!(
        r#match(b"hello world", b"hello", None),
        Ok(vec![b"hello".as_slice()])
    );
    assert_eq!(
        r#match(b"hello world", b"world", None),
        Ok(vec![b"world".as_slice()])
    );
    assert_eq!(r#match(b"hello world", b"bye", None), Ok(vec![]));
}

#[test]
fn test_pattern_classes() {
    assert_eq!(
        r#match(b"abc123", b"%a+", None),
        Ok(vec![b"abc".as_slice()])
    );
    assert_eq!(
        r#match(b"abc123", b"%d+", None),
        Ok(vec![b"123".as_slice()])
    );
    assert_eq!(
        r#match(b"abc123", b"[%D]+", None),
        Ok(vec![b"abc".as_slice()])
    );
    assert_eq!(
        r#match(b"abc123", b"[%A]+", None),
        Ok(vec![b"123".as_slice()])
    );
}

#[test]
fn test_single_capture() {
    assert_eq!(
        r#match(b"hello world", b"(hello)", None),
        Ok(vec![b"hello".as_slice()])
    );
}

#[test]
fn test_multiple_captures() {
    assert_eq!(
        r#match(b"hello world", b"(hello) (world)", None),
        Ok(vec![b"hello".as_slice(), b"world"])
    );
    assert_eq!(
        r#match(b"123-456-7890", b"(%d+)%-(%d+)%-(%d+)", None),
        Ok(vec![b"123".as_slice(), b"456", b"7890"])
    );
}

#[test]
fn test_combined_pattern_captures() {
    assert_eq!(
        r#match(b"abc123", b"(%a+)(%d+)", None),
        Ok(vec![b"abc".as_slice(), b"123"])
    );
}

#[test]
fn test_empty_captures() {
    assert_eq!(
        r#match(b"hello", b"(h)()ello", None),
        Ok(vec![b"h".as_slice(), b""])
    );
}

#[test]
fn test_init_parameter() {
    assert_eq!(
        r#match(b"hello world", b"world", Some(6)),
        Ok(vec![b"world".as_slice()])
    );
    assert_eq!(
        r#match(b"hello world", b"hello", Some(1)),
        Ok(vec![b"hello".as_slice()])
    );
    assert_eq!(r#match(b"hello world", b"hello", Some(2)), Ok(vec![]));
}

#[test]
fn test_empty_string_edge_cases() {
    assert_eq!(r#match(b"", b"", None), Ok(vec![b"".as_slice()]));
    assert_eq!(r#match(b"", b"^$", None), Ok(vec![b"".as_slice()]));
}

#[test]
fn test_anchor_patterns() {
    assert_eq!(r#match(b"hello", b"^", None), Ok(vec![b"".as_slice()]));
    assert_eq!(r#match(b"hello", b"$", None), Ok(vec![b"".as_slice()]));
    assert_eq!(
        r#match(b"hello", b"^hello$", None),
        Ok(vec![b"hello".as_slice()])
    );
}
