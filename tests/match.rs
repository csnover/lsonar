use lsonar::r#match;

fn convert_to_string_vec(items: &[&str]) -> Vec<String> {
    items.iter().map(|&s| s.to_string()).collect()
}

#[test]
fn test_simple_match() {
    assert_eq!(
        r#match("hello world", "hello", None),
        Ok(Some(convert_to_string_vec(&["hello"])))
    );
    assert_eq!(
        r#match("hello world", "world", None),
        Ok(Some(convert_to_string_vec(&["world"])))
    );
    assert_eq!(r#match("hello world", "bye", None), Ok(None));
}

#[test]
fn test_pattern_classes() {
    assert_eq!(
        r#match("abc123", "%a+", None),
        Ok(Some(convert_to_string_vec(&["abc"])))
    );
    assert_eq!(
        r#match("abc123", "%d+", None),
        Ok(Some(convert_to_string_vec(&["123"])))
    );
    assert_eq!(
        r#match("abc123", "[%D]+", None),
        Ok(Some(convert_to_string_vec(&["abc"])))
    );
    assert_eq!(
        r#match("abc123", "[%A]+", None),
        Ok(Some(convert_to_string_vec(&["123"])))
    );
}

#[test]
fn test_single_capture() {
    assert_eq!(
        r#match("hello world", "(hello)", None),
        Ok(Some(convert_to_string_vec(&["hello"])))
    );
}

#[test]
fn test_multiple_captures() {
    assert_eq!(
        r#match("hello world", "(hello) (world)", None),
        Ok(Some(convert_to_string_vec(&["hello", "world"])))
    );
    assert_eq!(
        r#match("123-456-7890", "(%d+)%-(%d+)%-(%d+)", None),
        Ok(Some(convert_to_string_vec(&["123", "456", "7890"])))
    );
}

#[test]
fn test_combined_pattern_captures() {
    assert_eq!(
        r#match("abc123", "(%a+)(%d+)", None),
        Ok(Some(convert_to_string_vec(&["abc", "123"])))
    );
}

#[test]
fn test_empty_captures() {
    assert_eq!(
        r#match("hello", "(h)()ello", None),
        Ok(Some(convert_to_string_vec(&["h", ""])))
    );
}

#[test]
fn test_init_parameter() {
    assert_eq!(
        r#match("hello world", "world", Some(6)),
        Ok(Some(convert_to_string_vec(&["world"])))
    );
    assert_eq!(
        r#match("hello world", "hello", Some(1)),
        Ok(Some(convert_to_string_vec(&["hello"])))
    );
    assert_eq!(r#match("hello world", "hello", Some(2)), Ok(None));
}

#[test]
fn test_empty_string_edge_cases() {
    assert_eq!(
        r#match("", "", None),
        Ok(Some(convert_to_string_vec(&[""])))
    );
    assert_eq!(
        r#match("", "^$", None),
        Ok(Some(convert_to_string_vec(&[""])))
    );
}

#[test]
fn test_anchor_patterns() {
    assert_eq!(
        r#match("hello", "^", None),
        Ok(Some(convert_to_string_vec(&[""])))
    );
    assert_eq!(
        r#match("hello", "$", None),
        Ok(Some(convert_to_string_vec(&[""])))
    );
    assert_eq!(
        r#match("hello", "^hello$", None),
        Ok(Some(convert_to_string_vec(&["hello"])))
    );
}
