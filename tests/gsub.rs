use lsonar::{gsub, Repl};
use std::collections::HashMap;

#[test]
fn test_basic_replacement() {
    assert_eq!(
        gsub(b"hello world", b"l", Repl::String(b"L"), None),
        Ok((b"heLLo worLd".to_vec(), 3))
    );
}

#[test]
fn test_limited_replacement_count() {
    assert_eq!(
        gsub(b"hello world", b"l", Repl::String(b"L"), Some(2)),
        Ok((b"heLLo world".to_vec(), 2))
    );
}

#[test]
fn test_zero_replacement_count() {
    assert_eq!(
        gsub(b"hello", b".", Repl::String(b"x"), Some(0)),
        Ok((b"hello".to_vec(), 0))
    );
}

#[test]
fn test_pattern_with_captures() {
    assert_eq!(
        gsub(
            b"name=John age=25",
            b"(%w+)=(%w+)",
            Repl::String(b"%2 is %1"),
            None
        ),
        Ok((b"John is name 25 is age".to_vec(), 2))
    );
}

#[test]
fn test_numeric_pattern() {
    assert_eq!(
        gsub(
            b"hello 123 world 456",
            b"%d+",
            Repl::String(b"<number>"),
            None
        ),
        Ok((b"hello <number> world <number>".to_vec(), 2))
    );
}

#[test]
fn test_empty_pattern() {
    assert_eq!(
        gsub(b"hello", b"", Repl::String(b"-"), None),
        Ok((b"-h-e-l-l-o-".to_vec(), 6))
    );
}

#[test]
fn test_escape_percent_in_replacement() {
    assert_eq!(
        gsub(b"hello", b"e", Repl::String(b"%% escaped"), None),
        Ok((b"h% escapedllo".to_vec(), 1))
    );
}

#[test]
fn test_complex_pattern_with_captures() {
    assert_eq!(
        gsub(
            b"User: John, Age: 25, Email: john@example.com",
            b"(User: )(%w+)(, Age: )(%d+)",
            Repl::String(b"%1%2%3%4 (adult)"),
            None
        ),
        Ok((
            b"User: John, Age: 25 (adult), Email: john@example.com".to_vec(),
            1
        ))
    );
}

#[test]
fn test_function_replacement() {
    assert_eq!(
        gsub(
            b"hello world",
            b"%w+",
            Repl::Function(Box::new(|captures: &[&[u8]]| {
                captures[0].to_ascii_uppercase()
            })),
            None
        ),
        Ok((b"HELLO WORLD".to_vec(), 2))
    );
}

#[test]
fn test_function_with_captures() {
    assert_eq!(
        gsub(
            b"a=1, b=2, c=3",
            b"(%w)=(%d)",
            Repl::Function(Box::new(|captures: &[&[u8]]| {
                format!(
                    "{}={}",
                    str::from_utf8(captures[1]).unwrap(),
                    str::from_utf8(captures[2]).unwrap().parse::<i32>().unwrap() * 2
                )
                .as_bytes()
                .to_vec()
            })),
            None
        ),
        Ok((b"a=2, b=4, c=6".to_vec(), 3))
    );
}

#[test]
fn test_table_replacement() {
    let mut table = HashMap::new();
    table.insert(b"hello".as_slice(), "привет".as_bytes());
    table.insert(b"world", "мир".as_bytes());

    assert_eq!(
        gsub(b"hello world", b"%w+", Repl::Table(&table), None),
        Ok(("привет мир".as_bytes().to_vec(), 2))
    );
}

#[test]
fn test_partial_table_replacement() {
    let mut table = HashMap::new();
    table.insert(b"hello".as_slice(), "привет".as_bytes());

    assert_eq!(
        gsub(b"hello world", b"%w+", Repl::Table(&table), None),
        Ok(("привет world".as_bytes().to_vec(), 2))
    );
}

#[test]
fn test_table_with_captures() {
    let mut table = HashMap::new();
    table.insert(b"name".as_slice(), "имя".as_bytes());
    table.insert(b"age", "возраст".as_bytes());

    assert_eq!(
        gsub(b"name=John age=25", b"(%w+)=%w+", Repl::Table(&table), None),
        Ok(("имя возраст".as_bytes().to_vec(), 2))
    );
}

#[test]
fn test_empty_string() {
    assert_eq!(
        gsub(b"", b"pattern", Repl::String(b"repl"), None),
        Ok((b"".to_vec(), 0))
    );
}

#[test]
fn test_pattern_not_found() {
    assert_eq!(
        gsub(b"hello", b"x", Repl::String(b"y"), None),
        Ok((b"hello".to_vec(), 0))
    );
}
