use lsonar::{Result, gmatch};

fn convert_to_string_vec(items: &[&str]) -> Vec<String> {
    items.iter().map(|&s| s.to_string()).collect()
}

fn collect_gmatch_results(text: &str, pattern: &str) -> Result<Vec<Vec<String>>> {
    let it = gmatch(text, pattern)?;
    it.collect()
}

#[test]
fn test_single_match() {
    assert_eq!(
        collect_gmatch_results("hello world", "hello"),
        Ok(vec![convert_to_string_vec(&["hello"])])
    );
}

#[test]
fn test_repeated_match() {
    assert_eq!(
        collect_gmatch_results("hello hello", "hello"),
        Ok(vec![
            convert_to_string_vec(&["hello"]),
            convert_to_string_vec(&["hello"])
        ])
    );
}

#[test]
fn test_numeric_pattern() {
    assert_eq!(
        collect_gmatch_results("abc123def456", "%d+"),
        Ok(vec![
            convert_to_string_vec(&["123"]),
            convert_to_string_vec(&["456"])
        ])
    );
}

#[test]
fn test_captures() {
    assert_eq!(
        collect_gmatch_results("name=John age=25", "(%a+)=(%w+)"),
        Ok(vec![
            convert_to_string_vec(&["name", "John"]),
            convert_to_string_vec(&["age", "25"])
        ])
    );
}

#[test]
fn test_single_char_captures() {
    assert_eq!(
        collect_gmatch_results("a=1 b=2 c=3", "(%a)=(%d)"),
        Ok(vec![
            convert_to_string_vec(&["a", "1"]),
            convert_to_string_vec(&["b", "2"]),
            convert_to_string_vec(&["c", "3"])
        ])
    );
}

#[test]
fn test_empty_captures() {
    assert_eq!(
        collect_gmatch_results("abc", "()a()"),
        Ok(vec![convert_to_string_vec(&["", ""])])
    );
}

#[test]
fn test_empty_pattern() {
    let result = collect_gmatch_results("abc", "").unwrap();
    assert_eq!(result.len(), 4);

    for r in &result {
        assert_eq!(r, &convert_to_string_vec(&[""]));
    }
}

#[test]
fn test_ip_address_pattern() {
    assert_eq!(
        collect_gmatch_results(
            "IPv4: 192.168.1.1 and 10.0.0.1",
            "(%d+)%.(%d+)%.(%d+)%.(%d+)"
        ),
        Ok(vec![
            convert_to_string_vec(&["192", "168", "1", "1"]),
            convert_to_string_vec(&["10", "0", "0", "1"])
        ])
    );
}

#[test]
fn test_html_tag_content() {
    assert_eq!(
        collect_gmatch_results("<p>First</p><p>Second</p>", "<p>([^<]+)</p>"),
        Ok(vec![
            convert_to_string_vec(&["First"]),
            convert_to_string_vec(&["Second"])
        ])
    );
}

#[test]
fn test_no_matches() {
    assert_eq!(
        collect_gmatch_results("hello world", "not found"),
        Ok(vec![])
    );
}

#[test]
fn test_empty_string() {
    assert_eq!(collect_gmatch_results("", "pattern"), Ok(vec![]));
}

#[test]
fn test_single_char_repeated() {
    assert_eq!(
        collect_gmatch_results("aaa", "a"),
        Ok(vec![
            convert_to_string_vec(&["a"]),
            convert_to_string_vec(&["a"]),
            convert_to_string_vec(&["a"])
        ])
    );
}

#[test]
fn test_dot_pattern() {
    let result = collect_gmatch_results("hello world", ".").unwrap();
    assert_eq!(result.len(), 11);

    for (i, v) in result.into_iter().enumerate() {
        assert_eq!(v[0], "hello world".chars().nth(i).unwrap().to_string());
    }
}
