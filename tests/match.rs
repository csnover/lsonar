use lsonar::r#match;

fn convert_to_string_vec(items: &[&[u8]]) -> Vec<Vec<u8>> {
    items.iter().map(|&s| s.to_vec()).collect()
}

#[test]
fn test_simple_match() {
    assert_eq!(
        r#match(b"hello world", b"hello", None),
        Ok(Some(vec![b"hello".to_vec()]))
    );
    assert_eq!(
        r#match(b"hello world", b"world", None),
        Ok(Some(vec![b"world".to_vec()]))
    );
    assert_eq!(r#match(b"hello world", b"bye", None), Ok(None));
}

#[test]
fn test_pattern_classes() {
    assert_eq!(
        r#match(b"abc123", b"%a+", None),
        Ok(Some(convert_to_string_vec(&[b"abc"])))
    );
    assert_eq!(
        r#match(b"abc123", b"%d+", None),
        Ok(Some(convert_to_string_vec(&[b"123"])))
    );
    assert_eq!(
        r#match(b"abc123", b"[%D]+", None),
        Ok(Some(convert_to_string_vec(&[b"abc"])))
    );
    assert_eq!(
        r#match(b"abc123", b"[%A]+", None),
        Ok(Some(convert_to_string_vec(&[b"123"])))
    );
}

#[test]
fn test_single_capture() {
    assert_eq!(
        r#match(b"hello world", b"(hello)", None),
        Ok(Some(convert_to_string_vec(&[b"hello"])))
    );
}

#[test]
fn test_multiple_captures() {
    assert_eq!(
        r#match(b"hello world", b"(hello) (world)", None),
        Ok(Some(convert_to_string_vec(&[b"hello", b"world"])))
    );
    assert_eq!(
        r#match(b"123-456-7890", b"(%d+)%-(%d+)%-(%d+)", None),
        Ok(Some(convert_to_string_vec(&[b"123", b"456", b"7890"])))
    );
}

#[test]
fn test_combined_pattern_captures() {
    assert_eq!(
        r#match(b"abc123", b"(%a+)(%d+)", None),
        Ok(Some(convert_to_string_vec(&[b"abc", b"123"])))
    );
}

#[test]
fn test_empty_captures() {
    assert_eq!(
        r#match(b"hello", b"(h)()ello", None),
        Ok(Some(convert_to_string_vec(&[b"h", b""])))
    );
}

#[test]
fn test_init_parameter() {
    assert_eq!(
        r#match(b"hello world", b"world", Some(6)),
        Ok(Some(convert_to_string_vec(&[b"world"])))
    );
    assert_eq!(
        r#match(b"hello world", b"hello", Some(1)),
        Ok(Some(convert_to_string_vec(&[b"hello"])))
    );
    assert_eq!(r#match(b"hello world", b"hello", Some(2)), Ok(None));
}

#[test]
fn test_empty_string_edge_cases() {
    assert_eq!(
        r#match(b"", b"", None),
        Ok(Some(convert_to_string_vec(&[b""])))
    );
    assert_eq!(
        r#match(b"", b"^$", None),
        Ok(Some(convert_to_string_vec(&[b""])))
    );
}

#[test]
fn test_anchor_patterns() {
    assert_eq!(
        r#match(b"hello", b"^", None),
        Ok(Some(convert_to_string_vec(&[b""])))
    );
    assert_eq!(
        r#match(b"hello", b"$", None),
        Ok(Some(convert_to_string_vec(&[b""])))
    );
    assert_eq!(
        r#match(b"hello", b"^hello$", None),
        Ok(Some(convert_to_string_vec(&[b"hello"])))
    );
}
