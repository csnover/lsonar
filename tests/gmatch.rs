use lsonar::{Result, gmatch};
use std::borrow::Cow;

fn collect_gmatch_results<'a>(text: &'a [u8], pattern: &[u8]) -> Result<Vec<Vec<Cow<'a, [u8]>>>> {
    let it = gmatch(text, pattern)?;
    Ok(it.collect())
}

#[test]
fn test_single_match() {
    assert_eq!(
        collect_gmatch_results(b"hello world", b"hello"),
        Ok(vec![vec![b"hello".into()]])
    );
}

#[test]
fn test_repeated_match() {
    assert_eq!(
        collect_gmatch_results(b"hello hello", b"hello"),
        Ok(vec![vec![b"hello".into()], vec![b"hello".into()]])
    );
}

#[test]
fn test_numeric_pattern() {
    assert_eq!(
        collect_gmatch_results(b"abc123def456", b"%d+"),
        Ok(vec![vec![b"123".into()], vec![b"456".into()]])
    );
}

#[test]
fn test_captures() {
    assert_eq!(
        collect_gmatch_results(b"name=John age=25", b"(%a+)=(%w+)"),
        Ok(vec![
            vec![b"name".into(), b"John".into()],
            vec![b"age".into(), b"25".into()]
        ])
    );
}

#[test]
fn test_single_char_captures() {
    assert_eq!(
        collect_gmatch_results(b"a=1 b=2 c=3", b"(%a)=(%d)"),
        Ok(vec![
            vec![b"a".into(), b"1".into()],
            vec![b"b".into(), b"2".into()],
            vec![b"c".into(), b"3".into()]
        ])
    );
}

#[test]
fn test_empty_captures() {
    assert_eq!(
        collect_gmatch_results(b"abc", b"()a()"),
        Ok(vec![vec![b"".into(), b"".into()]])
    );
}

#[test]
fn test_empty_pattern() {
    let result = collect_gmatch_results(b"abc", b"").unwrap();
    assert_eq!(result.len(), 4);

    for r in result {
        assert_eq!(r, vec![Cow::Borrowed(b"")]);
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
            vec![b"192".into(), b"168".into(), b"1".into(), b"1".into()],
            vec![b"10".into(), b"0".into(), b"0".into(), b"1".into()]
        ])
    );
}

#[test]
fn test_html_tag_content() {
    assert_eq!(
        collect_gmatch_results(b"<p>First</p><p>Second</p>", b"<p>([^<]+)</p>"),
        Ok(vec![vec![b"First".into()], vec![b"Second".into()]])
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
            vec![b"a".into()],
            vec![b"a".into()],
            vec![b"a".into()]
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
