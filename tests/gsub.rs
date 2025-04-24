use lsonar::{Repl, gsub};
use std::collections::HashMap;

#[test]
fn test_basic_replacement() {
    assert_eq!(
        gsub("hello world", "l", Repl::String("L"), None),
        Ok(("heLLo worLd".to_string(), 3))
    );
}

#[test]
fn test_limited_replacement_count() {
    assert_eq!(
        gsub("hello world", "l", Repl::String("L"), Some(2)),
        Ok(("heLLo world".to_string(), 2))
    );
}

#[test]
fn test_zero_replacement_count() {
    assert_eq!(
        gsub("hello", ".", Repl::String("x"), Some(0)),
        Ok(("hello".to_string(), 0))
    );
}

#[test]
fn test_pattern_with_captures() {
    assert_eq!(
        gsub(
            "name=John age=25",
            "(%w+)=(%w+)",
            Repl::String("%2 is %1"),
            None
        ),
        Ok(("John is name 25 is age".to_string(), 2))
    );
}

#[test]
fn test_numeric_pattern() {
    assert_eq!(
        gsub("hello 123 world 456", "%d+", Repl::String("<number>"), None),
        Ok(("hello <number> world <number>".to_string(), 2))
    );
}

#[test]
fn test_empty_pattern() {
    assert_eq!(
        gsub("hello", "", Repl::String("-"), None),
        Ok(("-h-e-l-l-o-".to_string(), 6))
    );
}

#[test]
fn test_escape_percent_in_replacement() {
    assert_eq!(
        gsub("hello", "e", Repl::String("%% escaped"), None),
        Ok(("h% escapedllo".to_string(), 1))
    );
}

#[test]
fn test_complex_pattern_with_captures() {
    assert_eq!(
        gsub(
            "User: John, Age: 25, Email: john@example.com",
            "(User: )(%w+)(, Age: )(%d+)",
            Repl::String("%1%2%3%4 (adult)"),
            None
        ),
        Ok((
            "User: John, Age: 25 (adult), Email: john@example.com".to_string(),
            1
        ))
    );
}

#[test]
fn test_function_replacement() {
    assert_eq!(
        gsub(
            "hello world",
            "%w+",
            Repl::Function(Box::new(|captures: &[&str]| { captures[0].to_uppercase() })),
            None
        ),
        Ok(("HELLO WORLD".to_string(), 2))
    );
}

#[test]
fn test_function_with_captures() {
    assert_eq!(
        gsub(
            "a=1, b=2, c=3",
            "(%w)=(%d)",
            Repl::Function(Box::new(|captures: &[&str]| {
                format!(
                    "{}={}",
                    captures[1],
                    captures[2].parse::<i32>().unwrap() * 2
                )
            })),
            None
        ),
        Ok(("a=2, b=4, c=6".to_string(), 3))
    );
}

#[test]
fn test_table_replacement() {
    let mut table = HashMap::new();
    table.insert("hello".to_string(), "привет".to_string());
    table.insert("world".to_string(), "мир".to_string());

    assert_eq!(
        gsub("hello world", "%w+", Repl::Table(&table), None),
        Ok(("привет мир".to_string(), 2))
    );
}

#[test]
fn test_partial_table_replacement() {
    let mut table = HashMap::new();
    table.insert("hello".to_string(), "привет".to_string());

    assert_eq!(
        gsub("hello world", "%w+", Repl::Table(&table), None),
        Ok(("привет world".to_string(), 2))
    );
}

#[test]
fn test_table_with_captures() {
    let mut table = HashMap::new();
    table.insert("name".to_string(), "имя".to_string());
    table.insert("age".to_string(), "возраст".to_string());

    assert_eq!(
        gsub("name=John age=25", "(%w+)=%w+", Repl::Table(&table), None),
        Ok(("имя возраст".to_string(), 2))
    );
}

#[test]
fn test_empty_string() {
    assert_eq!(
        gsub("", "pattern", Repl::String("repl"), None),
        Ok(("".to_string(), 0))
    );
}

#[test]
fn test_pattern_not_found() {
    assert_eq!(
        gsub("hello", "x", Repl::String("y"), None),
        Ok(("hello".to_string(), 0))
    );
}
