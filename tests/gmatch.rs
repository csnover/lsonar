use lsonar::{Result, gmatch};

fn collect_gmatch_results<'a>(text: &'a [u8], pattern: &[u8]) -> Result<Vec<Vec<&'a [u8]>>> {
    let it = gmatch(text, pattern)?;
    Ok(it.collect())
}

#[test]
fn test_single_match() {
    assert_eq!(
        collect_gmatch_results(b"hello world", b"hello"),
        Ok(vec![vec![b"hello".as_slice()]])
    );
}

#[test]
fn test_repeated_match() {
    assert_eq!(
        collect_gmatch_results(b"hello hello", b"hello"),
        Ok(vec![vec![b"hello".as_slice()], vec![b"hello".as_slice()]])
    );
}

#[test]
fn test_numeric_pattern() {
    assert_eq!(
        collect_gmatch_results(b"abc123def456", b"%d+"),
        Ok(vec![vec![b"123".as_slice()], vec![b"456".as_slice()]])
    );
}

#[test]
fn test_captures() {
    assert_eq!(
        collect_gmatch_results(b"name=John age=25", b"(%a+)=(%w+)"),
        Ok(vec![
            vec![b"name".as_slice(), b"John"],
            vec![b"age".as_slice(), b"25"]
        ])
    );
}

#[test]
fn test_single_char_captures() {
    assert_eq!(
        collect_gmatch_results(b"a=1 b=2 c=3", b"(%a)=(%d)"),
        Ok(vec![
            vec![b"a".as_slice(), b"1"],
            vec![b"b", b"2"],
            vec![b"c", b"3"]
        ])
    );
}

#[test]
fn test_empty_captures() {
    assert_eq!(
        collect_gmatch_results(b"abc", b"()a()"),
        Ok(vec![vec![b"".as_slice(), b""]])
    );
}

#[test]
fn test_empty_pattern() {
    let result = collect_gmatch_results(b"abc", b"").unwrap();
    assert_eq!(result.len(), 4);

    for r in result {
        assert_eq!(r, vec![b"".as_slice()]);
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
            vec![b"192".as_slice(), b"168", b"1", b"1"],
            vec![b"10", b"0", b"0", b"1"]
        ])
    );
}

#[test]
fn test_html_tag_content() {
    assert_eq!(
        collect_gmatch_results(b"<p>First</p><p>Second</p>", b"<p>([^<]+)</p>"),
        Ok(vec![vec![b"First".as_slice()], vec![b"Second"]])
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
        Ok(vec![vec![b"a".as_slice()], vec![b"a"], vec![b"a"]])
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
