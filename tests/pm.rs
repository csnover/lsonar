//! This file is a conversion of the Lua test suite pm.lua version 5.4.8
//! Copyright (C) 1994-2025 Lua.org, PUC-Rio.
//! SPDX-License-Identifier: MIT

use lsonar::{self as string, Repl};
use std::{borrow::Cow, collections::HashMap};

#[track_caller]
fn checkerror<F, T, E>(msg: &str, f: F)
where
    F: FnOnce() -> Result<T, E>,
    T: core::fmt::Debug,
    E: core::fmt::Display,
{
    let err = f().expect_err("should fail");
    assert!(
        format!("{err}").contains(msg),
        "'{err}' does not contain '{msg}'"
    );
}

#[track_caller]
fn f<'a>(s: &'a [u8], p: &[u8]) -> &'a [u8] {
    ok(string::find(s, p, None, false))
        .map(|result| &s[(result.start - 1)..result.end])
        .unwrap_or_default()
}

#[test]
fn find_basic() {
    let (a, b) = find(b"", b"", 0); // empty patterns are tricky
    assert_eq!((a, b), (1, 0));
    let (a, b) = find(b"alo", b"", 0);
    assert_eq!((a, b), (1, 0));
    let (a, b) = find(b"a\0o a\0o a\0o", b"a", 1); // first position
    assert_eq!((a, b), (1, 1));
    let (a, b) = find(b"a\0o a\0o a\0o", b"a\0o", 2); // starts in the midle
    assert_eq!((a, b), (5, 7));
    let (a, b) = find(b"a\0o a\0o a\0o", b"a\0o", 9); // starts in the midle
    assert_eq!((a, b), (9, 11));
    let (a, b) = find(b"a\0a\0a\0a\0\0ab", b"\0ab", 2); // finds at the end
    assert_eq!((a, b), (9, 11));
    let (a, b) = find(b"a\0a\0a\0a\0\0ab", b"b", 0); // last position
    assert_eq!((a, b), (11, 11));
    assert!(not_find(b"a\0a\0a\0a\0\0ab", b"b\0")); // check ending
    assert!(not_find(b"", b"\0"));
    assert_eq!(find(b"alo123alo", b"12", 0).0, 4);
    assert!(not_find(b"alo123alo", b"^12"));
}

#[test]
fn match_basic() {
    assert_eq!(r#match(b"aaab", b".*b"), &b"aaab"[..]);
    assert_eq!(r#match(b"aaa", b".*a"), &b"aaa"[..]);
    assert_eq!(r#match(b"b", b".*b"), &b"b"[..]);

    assert_eq!(r#match(b"aaab", b".+b"), &b"aaab"[..]);
    assert_eq!(r#match(b"aaa", b".+a"), &b"aaa"[..]);
    assert!(not_match(b"b", b".+b"));

    assert_eq!(r#match(b"aaab", b".?b"), &b"ab"[..]);
    assert_eq!(r#match(b"aaa", b".?a"), &b"aa"[..]);
    assert_eq!(r#match(b"b", b".?b"), &b"b"[..]);
}

#[test]
fn find_class_quantifier() {
    assert_eq!(f(b"aloALO", b"%l*"), b"alo");
    assert_eq!(f(b"aLo_ALO", b"%a*"), b"aLo");

    assert_eq!(f(b"  \n\r*&\n\r   xuxu  \n\n", b"%g%g%g+"), b"xuxu");
}

// Adapt a pattern to UTF-8
fn pu(p: &str) -> Vec<u8> {
    const UTF8_CHARPATTERN: &[u8] = b"[\0-\x7F\xC2-\xF4][\x80-\xBF]*";
    // reapply '?' into each individual byte of a character.
    // (For instance, "á?" becomes "\195?\161?".)
    let (p, ..) = ok(string::gsub(
        p.as_bytes(),
        &[b"(", UTF8_CHARPATTERN, b")%?"].concat(),
        Repl::Function(&mut |c| Some(ok(string::gsub(&c[0], b".", Repl::String(b"%0?"), None)).0)),
        None,
    ));
    // change '.' to utf-8 character patterns
    ok(string::gsub(
        &p,
        b"%.",
        Repl::String(UTF8_CHARPATTERN),
        None,
    ))
    .0
}

#[test]
fn find_literal_quantifier() {
    assert_eq!(f(b"aaab", b"a*"), b"aaa");
    assert_eq!(f(b"aaa", b"^.*$"), b"aaa");
    assert_eq!(f(b"aaa", b"b*"), b"");
    assert_eq!(f(b"aaa", b"ab*a"), b"aa");
    assert_eq!(f(b"aba", b"ab*a"), b"aba");
    assert_eq!(f(b"aaab", b"a+"), b"aaa");
    assert_eq!(f(b"aaa", b"^.+$"), b"aaa");
    assert!(f(b"aaa", b"b+").is_empty());
    assert!(f(b"aaa", b"ab+a").is_empty());
    assert_eq!(f(b"aba", b"ab+a"), b"aba");
    assert_eq!(f(b"a$a", b".$"), b"a");
    assert_eq!(f(b"a$a", b".%$"), b"a$");
    assert_eq!(f(b"a$a", b".$."), b"a$a");
}

#[test]
fn find_anchorlike() {
    assert!(f(b"a$a", b"$$").is_empty());
    assert!(f(b"a$b", b"a$").is_empty());
    assert_eq!(f(b"a$a", b"$"), b"");
}

#[test]
fn find_zero_quantifier() {
    assert_eq!(f(b"", b"b*"), b"");
    assert!(f(b"aaa", b"bb*").is_empty());
}

#[test]
fn find_non_greedy() {
    assert_eq!(f(b"aaab", b"a-"), b"");
    assert_eq!(f(b"aaa", b"^.-$"), b"aaa");
    assert_eq!(f(b"aabaaabaaabaaaba", b"b.*b"), b"baaabaaabaaab");
    assert_eq!(f(b"aabaaabaaabaaaba", b"b.-b"), b"baaab");
}

#[test]
fn find_end_anchor() {
    assert_eq!(f(b"alo xo", b".o$"), b"xo");
}

#[test]
fn find_unicode_class() {
    assert_eq!(f(" \n isto é assim".as_bytes(), b"%S%S*"), b"isto");
    assert_eq!(f(" \n isto é assim".as_bytes(), b"%S*$"), b"assim");
    assert_eq!(f(" \n isto é assim".as_bytes(), b"[a-z]*$"), b"assim");
}

#[test]
fn find_question_mark() {
    assert_eq!(f(b"um caracter ? extra", b"[^%sa-z]"), b"?");
    assert_eq!(f(b"", b"a?"), b"");
}

#[test]
fn find_unicode() {
    assert_eq!(f("á".as_bytes(), &pu("á?")), "á".as_bytes());
    assert_eq!(f("ábl".as_bytes(), &pu("á?b?l?")), "ábl".as_bytes());
    assert_eq!(f("  ábl".as_bytes(), &pu("á?b?l?")), b"");
}

#[test]
fn find_edge_cases() {
    assert_eq!(f(b"aa", b"^aa?a?a"), b"aa");
    assert_eq!(f("]]]áb".as_bytes(), b"[^]]+"), "áb".as_bytes());
    assert_eq!(f(b"0alo alo", b"%x*"), b"0a");
    assert_eq!(f(b"alo alo", b"%C+"), b"alo alo");
}

#[test]
fn gsub_dynamic_capture() {
    #[track_caller]
    fn f1<'a>(s: &'a [u8], p: &[u8]) -> &'a [u8] {
        let (p, ..) = ok(string::gsub(
            p,
            b"%%([0-9])",
            Repl::Function(&mut |s| {
                Some(
                    [
                        b"%",
                        (to_number::<i32>(&s[0]) + 1i32).to_string().as_bytes(),
                    ]
                    .concat(),
                )
            }),
            None,
        ));
        let (p, ..) = ok(string::gsub(&p, b"^(^?)", Repl::String(b"%1()"), Some(1)));
        let (p, ..) = ok(string::gsub(&p, b"($?)$", Repl::String(b"()%1"), Some(1)));
        let t = string::r#match(s, &p, None).expect("matched");
        &s[(to_number::<usize>(&t[1]) - 1)..=to_number::<usize>(t.last().unwrap())]
    }

    assert_eq!(f1(b"alo alx 123 b\0o b\0o", b"(..*) %1"), &b"b\0o b\0o"[..]);
    assert_eq!(
        f1(b"axz123= 4= 4 34", b"(.+)=(.*)=%2 %1"),
        &b"3= 4= 4 3"[..]
    );
    assert_eq!(f1(b"=======", b"^(=*)=%1$"), &b"======="[..]);
    assert!(ok(string::r#match(b"==========", b"^([=]*)=%1$", None)).is_empty());
}

#[test]
fn gsub_strset() {
    #[track_caller]
    fn strset(p: &[u8]) -> Vec<u8> {
        const ABC: &[u8; 256] = &const {
            let mut abc = [0u8; 256];
            let mut i = 0;
            while i < 256 {
                abc[i] = i as u8;
                i += 1;
            }
            abc
        };

        let mut result = Vec::<u8>::new();
        ok(string::gsub(
            ABC,
            p,
            Repl::Function(&mut |c| {
                result.extend(c[0].as_ref());
                None
            }),
            None,
        ));
        result
    }

    assert_eq!(strset(b"[\xc8-\xd2]").len(), 11);

    assert_eq!(strset(b"[a-z]"), b"abcdefghijklmnopqrstuvwxyz");
    assert_eq!(strset(b"[a-z%d]"), strset(b"[%da-uu-z]"));
    assert_eq!(strset(b"[a-]"), b"-a");
    assert_eq!(strset(b"[^%W]"), strset(b"[%w]"));
    assert_eq!(strset(b"[]%%]"), b"%]");
    assert_eq!(strset(b"[a%-z]"), b"-az");
    assert_eq!(strset(b"[%^%[%-a%]%-b]"), b"-[]^ab");
    assert_eq!(strset(b"%Z"), strset(b"[\x01-\xff]"));
    assert_eq!(strset(b"."), strset(b"[\x01-\xff%z]"));
}

#[test]
fn match_capture() {
    #[track_caller]
    fn to_refs<'a, 'b>(s: &'a [Cow<'b, [u8]>]) -> Vec<&'a [u8]> {
        s.iter().map(|v| v.as_ref()).collect()
    }

    assert_eq!(r#match(b"alo xyzK", b"(%w+)K"), &b"xyz"[..]);
    assert_eq!(r#match(b"254 K", b"(%d*)K"), &b""[..]);
    assert_eq!(r#match(b"alo ", b"(%w*)$"), &b""[..]);
    assert!(not_match(b"alo ", b"(%w+)$"));
    assert_eq!(find("(álo)".as_bytes(), "%(á".as_bytes(), 0).0, 1);
    let abcde = ok(string::r#match(
        "âlo alo".as_bytes(),
        &pu("^(((.).). (%w*))$"),
        None,
    ));
    assert_eq!(
        [
            "âlo alo".as_bytes(),
            "âl".as_bytes(),
            "â".as_bytes(),
            b"alo"
        ],
        to_refs(&abcde).as_slice()
    );
    let abcd = ok(string::r#match(b"0123456789", b"(.+(.?)())", None));
    assert_eq!([&b"0123456789"[..], b"", b"11"], to_refs(&abcd).as_slice());
}

#[test]
fn gsub_unicode() {
    assert_eq!(gsub("ülo ülo".as_bytes(), "ü".as_bytes(), b"x"), b"xlo xlo");
}

#[test]
fn gsub_trim() {
    assert_eq!(
        gsub("alo úlo  ".as_bytes(), b" +$", b""),
        "alo úlo".as_bytes()
    ); // trim
    assert_eq!(gsub(b"  alo alo  ", b"^%s*(.-)%s*$", b"%1"), b"alo alo"); // double trim
    assert_eq!(gsub(b"alo  alo  \n 123\n ", b"%s+", b" "), b"alo alo 123 ");
}

#[test]
fn gsub_capture() {
    let t = "abç d";
    let (a, b) = ok(string::gsub(
        t.as_bytes(),
        &pu("(.)"),
        Repl::String(b"%1@"),
        None,
    ));
    assert_eq!((a.as_slice(), b), ("a@b@ç@ @d@".as_bytes(), b));
    let (a, b) = ok(string::gsub(
        "abçd".as_bytes(),
        &pu("(.)"),
        Repl::String(b"%0@"),
        Some(2),
    ));
    assert_eq!((a.as_slice(), b), ("a@b@çd".as_bytes(), 2));
    assert_eq!(gsub(b"alo alo", b"()[al]", b"%1"), b"12o 56o");
    assert!(gsub(b"abc=xyz", b"(%w*)(%p)(%w+)", b"%3%2%1-%0") == b"xyz=abc-abc=xyz");
    assert_eq!(gsub(b"abc", b"%w", b"%1%0"), b"aabbcc");
    assert_eq!(gsub(b"abc", b"%w+", b"%0%1"), b"abcabc");
    assert_eq!(
        gsub("áéí".as_bytes(), b"$", "\0óú".as_bytes()),
        "áéí\0óú".as_bytes()
    );
    assert_eq!(gsub(b"", b"^", b"r"), b"r");
    assert_eq!(gsub(b"", b"$", b"r"), b"r");
}

#[test]
fn gsub_empty_533() {
    // new (5.3.3) semantics for empty matches
    assert_eq!(gsub(b"a b cd", b" *", b"-"), b"-a-b-c-d-");

    let mut res = Vec::new();
    let sub = b"a  \nbc\t\td";
    let mut i = 1;
    for cap in ok(string::gmatch(sub, b"()%s*()", None)) {
        res.extend(&sub[i - 1..to_number::<usize>(&cap[0]) - 1]);
        res.push(b'-');
        i = to_number::<usize>(&cap[1]);
    }
    // XXX: The unit test in the Lua test suite has an output which does not
    // match PUC-Lua. It is changed here to match PUC-Lua, since this matches
    // our output too.
    // assert_eq!(res, b"-a-b-c-d-");
    assert_eq!(res, b"-a--b-c--d-");
}

#[test]
fn gsub_nested_capture_literal() {
    assert!(
        ok(string::gsub(
            b"um (dois) tres (quatro)",
            b"(%(%w+%))",
            Repl::Function(&mut |c| { Some(c[0].to_ascii_uppercase()) }),
            None
        ))
        .0 == b"um (DOIS) tres (QUATRO)"
    );
}

#[test]
fn gsub_fn() {
    let mut globals = HashMap::new();
    ok(string::gsub(
        b"a=roberto,roberto=a",
        b"(%w+)=(%w%w*)",
        Repl::Function(&mut |caps| {
            let (n, v) = (&caps[0], &caps[1]);
            globals.insert(n.to_vec(), v.to_vec());
            None
        }),
        None,
    ));
    assert_eq!(
        globals.get(&b"a"[..]).map(|x| x.as_slice()),
        Some(&b"roberto"[..])
    );
    assert_eq!(
        globals.get(&b"roberto"[..]).map(|x| x.as_slice()),
        Some(&b"a"[..])
    );

    fn f(a: &[lsonar::Capture<'_>]) -> Option<Vec<u8>> {
        Some(gsub(&a[0], b".", &a[1]))
    }
    assert!(
        ok(string::gsub(
            "trocar tudo em |teste|b| é |beleza|al|".as_bytes(),
            b"|([^|]*)|([^|]*)|",
            Repl::Function(&mut f),
            None
        ))
        .0 == "trocar tudo em bbbbb é alalalalalal".as_bytes()
    );

    // XXX: This cannot be tested without a Lua VM.
    // local function dostring (s) return load(s, "")() or "" end
    // assert!(string::gsub("alo $a='x'$ novamente $return a$",
    //                 "$([^$]*)%$",
    //                 dostring) == "alo  novamente x")

    // local x = string::gsub("$x=string::gsub('alo', '.', string.upper)$ assim vai para $return x$",
    //         "$([^$]*)%$", dostring)
    // assert!(x == b" assim vai para ALO")
    // _G.a, _G.x = nil
}

#[test]
fn gsub_position_capture() {
    let mut t = HashMap::new();
    let s = b"a alo jose  joao";
    let r = ok(string::gsub(
        s,
        b"()(%w+)()",
        Repl::Function(&mut |caps| {
            let (a, w, b) = (&caps[0], &caps[1], &caps[2]);
            let a = to_number::<usize>(a);
            let len = to_number::<usize>(b) - a;
            assert_eq!(w.len(), len);
            t.insert(a, len);
            None
        }),
        None,
    ))
    .0;
    assert_eq!(s[..], r);
    assert_eq!(t[&1], 1);
    assert_eq!(t[&3], 3);
    assert_eq!(t[&7], 4);
    assert_eq!(t[&13], 4);
}

#[test]
fn gsub_balanced() {
    fn isbalanced(s: &[u8]) -> bool {
        ok(string::find(&gsub(s, b"%b()", b""), b"[()]", None, false)).is_none()
    }

    assert!(isbalanced(b"(9 ((8))(\0) 7) \0\0 a b ()(c)() a"));
    assert!(!isbalanced(b"(9 ((8) 7) a b (\0 c) a"));
    assert_eq!(gsub(b"alo 'oi' alo", b"%b''", b"\""), b"alo \" alo");
}

#[test]
fn gsub_fn_count() {
    let t = [&b"apple"[..], b"orange", b"lime"];
    let mut n = 0;
    assert_eq!(
        ok(string::gsub(
            b"x and x and x",
            b"x",
            Repl::Function(&mut |_| {
                n += 1;
                Some(t[n - 1].to_vec())
            }),
            None
        ))
        .0,
        b"apple and orange and lime"
    );

    let mut t = vec![];
    let _ = ok(string::gsub(
        b"first second word",
        b"%w%w*",
        Repl::Function(&mut |w| {
            t.push(w[0].to_vec());
            None
        }),
        None,
    ));
    assert_eq!(t.as_slice(), &[&b"first"[..], b"second", b"word"]);

    let mut t = vec![];
    assert_eq!(
        ok(string::gsub(
            b"first second word",
            b"%w+",
            Repl::Function(&mut |w| {
                t.push(w[0].to_vec());
                None
            }),
            Some(2)
        ))
        .0,
        b"first second word"
    );
    assert_eq!(t.as_slice(), &[&b"first"[..], b"second"]);
}

#[test]
fn gsub_errors() {
    // XXX: This kind of type confusion is impossible in Rust
    // checkerror("invalid replacement value %(a table%)",
    //             string::gsub, "alo", ".", {a = {}})
    checkerror("invalid capture index %%2", || {
        string::gsub(b"alo", b".", Repl::String(b"%2"), None)
    });
    checkerror("invalid capture index %%0", || {
        string::gsub(b"alo", b"(%0)", Repl::String(b"a"), None)
    });
    checkerror("invalid capture index %%1", || {
        string::gsub(b"alo", b"(%1)", Repl::String(b"a"), None)
    });
    checkerror("invalid use of '%%'", || {
        string::gsub(b"alo", b".", Repl::String(b"%x"), None)
    });
}

#[test]
fn find_big_strings() {
    // if not _soft then
    // print("big strings")
    let a = core::iter::repeat_n(b'a', 300000).collect::<Vec<_>>();
    find(&a, b"^a*.?$", 0);
    assert!(not_find(&a, b"^a*.?b$"));
    find(&a, b"^a-.?$", 0);

    // XXX: No need to test PUC-Lua implementation bugs
    // bug in 5.1.2
    // a = string.rep(b"a", 10000) .. string.rep(b"b", 10000)
    // assert!(not pcall(string::gsub, a, b"b"))
    // end
}

#[test]
fn gsub_recursive() {
    // recursive nest of gsubs
    fn rev(s: &[u8]) -> Vec<u8> {
        ok(string::gsub(
            s,
            b"(.)(.+)",
            Repl::Function(&mut |c| {
                let mut r = rev(&c[1]);
                r.extend(c[0].iter());
                Some(r)
            }),
            None,
        ))
        .0
    }

    let x = b"abcdef";
    assert_eq!(rev(&rev(x)), x);
}

#[test]
fn gsub_tables() {
    // gsub with tables
    assert_eq!(
        ok(string::gsub(b"alo alo", b".", Repl::Table(&|_| None), None)).0,
        b"alo alo"
    );
    assert!(
        ok(string::gsub(
            b"alo alo",
            b"(.)",
            Repl::Table(&|k| match k.as_ref() {
                b"a" => Some(b"AA".to_vec()),
                b"l" => Some(b"".to_vec()),
                _ => None,
            }),
            None
        ))
        .0 == b"AAo AAo"
    );
    assert!(
        ok(string::gsub(
            b"alo alo",
            b"(.).",
            Repl::Table(&|k| match k.as_ref() {
                b"a" => Some(b"AA".to_vec()),
                b"l" => Some(b"K".to_vec()),
                _ => None,
            }),
            None
        ))
        .0 == b"AAo AAo"
    );
    assert!(
        ok(string::gsub(
            b"alo alo",
            b"((.)(.?))",
            Repl::Table(&|k| match k.as_ref() {
                b"al" => Some(b"AA".to_vec()),
                // XXX: This kind of type confusion is impossible in Rust
                b"o" => None,
                _ => None,
            }),
            None
        ))
        .0 == b"AAo AAo"
    );

    assert!(
        ok(string::gsub(
            b"alo alo",
            b"().",
            Repl::Table(&|k| match k.as_ref() {
                b"1" => Some(b"x".to_vec()),
                b"2" => Some(b"yy".to_vec()),
                b"3" => Some(b"zzz".to_vec()),
                _ => None,
            }),
            None
        ))
        .0 == b"xyyzzz alo"
    );

    assert!(
        ok(string::gsub(
            b"a alo b hi",
            b"%w%w+",
            Repl::Table(&|k| Some(k.to_ascii_uppercase())),
            None
        ))
        .0 == b"a ALO b HI"
    );
}

#[test]
fn gmatch_basic() {
    // tests for gmatch
    let mut a = 0;
    for i in ok(string::gmatch(b"abcde", b"()", None)) {
        let i = to_number::<i32>(&i[0]);
        assert_eq!(i, a + 1);
        a = i;
    }
    assert_eq!(a, 6);

    let mut t = vec![];
    for w in ok(string::gmatch(b"first second word", b"%w+", None)) {
        t.push(w[0].to_vec());
    }
    assert_eq!(t, [&b"first"[..], b"second", b"word"]);

    let mut t = vec![3, 6, 9];
    for i in ok(string::gmatch(b"xuxx uu ppar r", b"()(.)%2", None)) {
        assert_eq!(to_number::<i32>(&i[0]), t.remove(0));
    }
    assert!(t.is_empty());

    let mut t = HashMap::new();
    for ij in ok(string::gmatch(
        b"13 14 10 = 11, 15= 16, 22=23",
        b"(%d+)%s*=%s*(%d+)",
        None,
    )) {
        let (i, j) = (&ij[0], &ij[1]);
        t.insert(to_number::<i32>(i), to_number::<i32>(j));
    }
    let mut a = 0;
    for (k, v) in t.iter() {
        assert_eq!(*k + 1, *v);
        a += 1;
    }
    assert_eq!(a, 3);
}

#[test]
fn gmatch_init_param() {
    // init parameter in gmatch
    let mut s = 0;
    for k in ok(string::gmatch(b"10 20 30", b"%d+", Some(3))) {
        s += to_number::<i32>(&k[0]);
    }
    assert_eq!(s, 50);

    let mut s = 0;
    for k in ok(string::gmatch(b"11 21 31", b"%d+", Some(-4))) {
        s += to_number::<i32>(&k[0]);
    }
    assert_eq!(s, 32);

    // there is an empty string at the end of the subject
    let mut s = 0;
    for _k in ok(string::gmatch(b"11 21 31", b"%w*", Some(9))) {
        s += 1;
    }
    assert_eq!(s, 1);

    // there are no empty strings after the end of the subject
    let mut s = 0;
    for _k in ok(string::gmatch(b"11 21 31", b"%w*", Some(10))) {
        s += 1;
    }
    assert_eq!(s, 0);
}

#[test]
fn pattern_frontiers() {
    // tests for `%f' (`frontiers')

    assert_eq!(gsub(b"aaa aa a aaa a", b"%f[%w]a", b"x"), b"xaa xa x xaa x");
    assert_eq!(gsub(b"[[]] [][] [[[[", b"%f[[].", b"x"), b"x[]] x]x] x[[[");
    assert_eq!(gsub(b"01abc45de3", b"%f[%d]", b"."), b".01abc.45de.3");
    assert_eq!(gsub(b"01abc45 de3x", b"%f[%D]%w", b"."), b"01.bc45 de3.");
    assert_eq!(gsub(b"function", b"%f[\x01-\xff]%w", b"."), b".unction");
    assert_eq!(gsub(b"function", b"%f[^\x01-\xff]", b"."), b"function.");

    assert_eq!(find(b"a", b"%f[a]", 0).0, 1);
    assert_eq!(find(b"a", b"%f[^%z]", 0).0, 1);
    assert_eq!(find(b"a", b"%f[^%l]", 0).0, 2);
    assert_eq!(find(b"aba", b"%f[a%z]", 0).0, 3);
    assert_eq!(find(b"aba", b"%f[%z]", 0).0, 4);
    assert!(not_find(b"aba", b"%f[%l%z]"));
    assert!(not_find(b"aba", b"%f[^%l%z]"));

    let (i, e) = find(b" alo aalo allo", b"%f[%S].-%f[%s].-%f[%S]", 0);
    assert!(i == 2 && e == 5);
    let k = r#match(b" alo aalo allo", b"%f[%S](.-%f[%s].-%f[%S])");
    assert_eq!(&k[..], b"alo ");

    let mut a = vec![1, 5, 9, 14, 17];
    for k in ok(string::gmatch(b"alo alo th02 is 1hat", b"()%f[%w%d]", None)) {
        assert_eq!(a.remove(0), to_number::<i32>(&k[0]));
    }
    assert!(a.is_empty());
}

#[test]
fn pattern_malformed() {
    // malformed patterns
    #[track_caller]
    fn malform(p: &[u8], mut m: &str) {
        if m.is_empty() {
            m = "malformed";
        }

        checkerror(m, || string::find(b"a", p, None, false));
    }

    malform(b"(.", "unfinished capture");
    malform(b".)", "invalid pattern capture");
    malform(b"[a", "");
    malform(b"[]", "");
    malform(b"[^]", "");
    malform(b"[a%]", "");
    malform(b"[a%", "");
    malform(b"%b", "");
    malform(b"%ba", "");
    malform(b"%", "");
    malform(b"%f", "missing");
}

#[test]
fn pattern_null() {
    // \0 in patterns
    assert_eq!(r#match(b"ab\0\x01\x02c", b"[\0-\x02]+"), &b"\0\x01\x02"[..]);
    assert_eq!(r#match(b"ab\0\x01\x02c", b"[\0-\0]+"), &b"\0"[..]);
    assert_eq!(find(b"b$a", b"$\0?", 0).0, 2);
    assert_eq!(find(b"abc\0efg", b"%\0", 0).0, 4);
    assert_eq!(
        r#match(b"abc\0efg\0\x01e\x01g", b"%b\0\x01"),
        &b"\0efg\0\x01e\x01"[..]
    );
    assert_eq!(r#match(b"abc\0\0\0", b"%\0+"), &b"\0\0\0"[..]);
    assert_eq!(r#match(b"abc\0\0\0", b"%\0%\0?"), &b"\0\0"[..]);

    // magic char after \0
    assert_eq!(find(b"abc\0\0", b"\0.", 0).0, 4);
    assert_eq!(find(b"abcx\0\0abc\0abc", b"x\0\0abc\0a.", 0).0, 4);
}

#[test]
fn gsub_reuse() {
    // test reuse of original string in gsub
    let s = core::iter::repeat_n(b'a', 100).collect::<Vec<_>>();
    // XXX: This is a Lua VM implementation detail.
    // let r = gsub(&s, b"b", b"c");   // no match
    // assert!(string.format("%p", s) == string.format("%p", r));

    // r = string::gsub(s, ".", {x = "y"})   // no substitutions
    // assert!(string.format("%p", s) == string.format("%p", r))

    let mut count = 0;
    let _r = ok(string::gsub(
        &s,
        b".",
        Repl::Function(&mut |x| {
            assert_eq!(x[0], &b"a"[..]);
            count += 1;
            None // no substitution
        }),
        None,
    ));
    // r = string::gsub(r, ".", {b = b"x"})   // "a" is not a key; no subst.
    assert_eq!(count, 100);
    // assert!(string.format("%p", s) == string.format("%p", r))

    let mut count = 0;
    let _r = ok(string::gsub(
        &s,
        b".",
        Repl::Function(&mut |x| {
            assert_eq!(x[0], &b"a"[..]);
            count += 1;
            Some(x[0].to_vec()) // substitution...
        }),
        None,
    ));
    assert_eq!(count, 100);
    // no reuse in this case
    // assert!(r == s and string.format("%p", s) ~= string.format("%p", r))
}

// The rest of this file are Rust-specific convenience functions

#[track_caller]
fn ok<T, E>(r: Result<T, E>) -> T
where
    E: core::fmt::Debug,
{
    r.expect("should accept pattern string")
}

#[track_caller]
fn gsub(s: &[u8], p: &[u8], r: &[u8]) -> Vec<u8> {
    ok(string::gsub(s, p, Repl::String(r), None)).0
}

#[track_caller]
fn find(s: &[u8], pattern: &[u8], init: isize) -> (usize, usize) {
    let result = ok(string::find(
        s,
        pattern,
        if init == 0 { None } else { Some(init) },
        false,
    ))
    .expect("should find match");
    (result.start, result.end)
}

#[track_caller]
fn not_find(s: &[u8], pattern: &[u8]) -> bool {
    ok(string::find(s, pattern, None, false)).is_none()
}

#[track_caller]
fn r#match<'a>(s: &'a [u8], pattern: &[u8]) -> Cow<'a, [u8]> {
    ok(string::r#match(s, pattern, None))
        .into_iter()
        .next()
        .expect("should find match")
}

#[track_caller]
fn not_match(s: &[u8], pattern: &[u8]) -> bool {
    ok(string::r#match(s, pattern, None)).is_empty()
}

#[track_caller]
fn to_number<T>(s: &[u8]) -> T
where
    T: core::str::FromStr,
    <T as core::str::FromStr>::Err: core::fmt::Debug,
{
    str::from_utf8(s)
        .expect("should be valid utf-8")
        .parse::<T>()
        .expect("should be a valid integer")
}
