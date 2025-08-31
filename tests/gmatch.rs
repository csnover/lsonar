use lsonar::{Result, gmatch};

fn convert_to_string_vec(items: &[&[u8]]) -> Vec<Vec<u8>> {
    items.iter().map(|&s| s.to_vec()).collect()
}

fn collect_gmatch_results(text: &[u8], pattern: &[u8]) -> Result<Vec<Vec<Vec<u8>>>> {
    let it = gmatch(text, pattern)?;
    it.collect()
}

#[test]
fn test_single_match() {
    assert_eq!(
        collect_gmatch_results(b"hello world", b"hello"),
        Ok(vec![convert_to_string_vec(&[b"hello"])])
    );
}

#[test]
fn test_repeated_match() {
    assert_eq!(
        collect_gmatch_results(b"hello hello", b"hello"),
        Ok(vec![
            convert_to_string_vec(&[b"hello"]),
            convert_to_string_vec(&[b"hello"])
        ])
    );
}

#[test]
fn test_numeric_pattern() {
    assert_eq!(
        collect_gmatch_results(b"abc123def456", b"%d+"),
        Ok(vec![
            convert_to_string_vec(&[b"123"]),
            convert_to_string_vec(&[b"456"])
        ])
    );
}

#[test]
fn test_captures() {
    assert_eq!(
        collect_gmatch_results(b"name=John age=25", b"(%a+)=(%w+)"),
        Ok(vec![
            convert_to_string_vec(&[b"name", b"John"]),
            convert_to_string_vec(&[b"age", b"25"])
        ])
    );
}

#[test]
fn test_single_char_captures() {
    assert_eq!(
        collect_gmatch_results(b"a=1 b=2 c=3", b"(%a)=(%d)"),
        Ok(vec![
            convert_to_string_vec(&[b"a", b"1"]),
            convert_to_string_vec(&[b"b", b"2"]),
            convert_to_string_vec(&[b"c", b"3"])
        ])
    );
}

#[test]
fn test_empty_captures() {
    assert_eq!(
        collect_gmatch_results(b"abc", b"()a()"),
        Ok(vec![convert_to_string_vec(&[b"", b""])])
    );
}

#[test]
fn test_empty_pattern() {
    let result = collect_gmatch_results(b"abc", b"").unwrap();
    assert_eq!(result.len(), 4);

    for r in &result {
        assert_eq!(r, &convert_to_string_vec(&[b""]));
    }
}

#[test]
fn test_ip_address_pattern() {
    assert_eq!(
        collect_gmatch_results(
            b"IPv4: 192.168.1.1 and 10.0.0.1",
            b"(%d+)%.(%d+)%.(%d+)%.(%d+)"
        ),
        Ok(vec![
            convert_to_string_vec(&[b"192", b"168", b"1", b"1"]),
            convert_to_string_vec(&[b"10", b"0", b"0", b"1"])
        ])
    );
}

#[test]
fn test_html_tag_content() {
    assert_eq!(
        collect_gmatch_results(b"<p>First</p><p>Second</p>", b"<p>([^<]+)</p>"),
        Ok(vec![
            convert_to_string_vec(&[b"First"]),
            convert_to_string_vec(&[b"Second"])
        ])
    );
}

#[test]
fn test_no_matches() {
    assert_eq!(
        collect_gmatch_results(b"hello world", b"not found"),
        Ok(vec![])
    );
}

#[test]
fn test_empty_string() {
    assert_eq!(collect_gmatch_results(b"", b"pattern"), Ok(vec![]));
}

#[test]
fn test_single_char_repeated() {
    assert_eq!(
        collect_gmatch_results(b"aaa", b"a"),
        Ok(vec![
            convert_to_string_vec(&[b"a"]),
            convert_to_string_vec(&[b"a"]),
            convert_to_string_vec(&[b"a"])
        ])
    );
}

#[test]
fn test_dot_pattern() {
    let result = collect_gmatch_results(b"hello world", b".").unwrap();
    assert_eq!(result.len(), 11);

    for (i, v) in result.into_iter().enumerate() {
        assert_eq!(v[0], vec![b"hello world"[i]]);
    }
}
