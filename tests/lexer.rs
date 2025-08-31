use lsonar::{
    lexer::{token::Token, Lexer},
    Error, Result,
};

fn lex_all(input: &[u8]) -> Result<Vec<Token>> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    while let Some(token_result) = lexer.next_token()? {
        tokens.push(token_result);
    }
    Ok(tokens)
}

#[test]
fn test_basic_tokens_lexer() -> Result<()> {
    assert_eq!(
        lex_all(b"abc")?,
        vec![
            Token::Literal(b'a'),
            Token::Literal(b'b'),
            Token::Literal(b'c')
        ]
    );
    assert_eq!(
        lex_all(b"a.c")?,
        vec![Token::Literal(b'a'), Token::Any, Token::Literal(b'c')]
    );
    assert_eq!(lex_all(b"()")?, vec![Token::LParen, Token::RParen]);
    assert_eq!(lex_all(b"[]")?, vec![Token::LBracket, Token::RBracket]);
    assert_eq!(
        lex_all(b"^$*+?-")?,
        vec![
            Token::Caret,
            Token::Dollar,
            Token::Star,
            Token::Plus,
            Token::Question,
            Token::Minus
        ]
    );
    Ok(())
}

#[test]
fn test_escape_tokens_lexer() -> Result<()> {
    assert_eq!(lex_all(b"%%")?, vec![Token::EscapedLiteral(b'%')]);
    assert_eq!(
        lex_all(b"%.%a")?,
        vec![Token::EscapedLiteral(b'.'), Token::Class(b'a')]
    );
    assert_eq!(lex_all(b"%(")?, vec![Token::EscapedLiteral(b'(')]);
    assert_eq!(lex_all(b"%)")?, vec![Token::EscapedLiteral(b')')]);
    assert_eq!(lex_all(b"%[")?, vec![Token::EscapedLiteral(b'[')]);
    assert_eq!(lex_all(b"%]")?, vec![Token::EscapedLiteral(b']')]);
    assert_eq!(lex_all(b"%*")?, vec![Token::EscapedLiteral(b'*')]);
    assert_eq!(lex_all(b"%+")?, vec![Token::EscapedLiteral(b'+')]);
    assert_eq!(lex_all(b"%?")?, vec![Token::EscapedLiteral(b'?')]);
    assert_eq!(lex_all(b"%-")?, vec![Token::EscapedLiteral(b'-')]);
    assert_eq!(lex_all(b"%^")?, vec![Token::EscapedLiteral(b'^')]);
    assert_eq!(lex_all(b"%$")?, vec![Token::EscapedLiteral(b'$')]);
    Ok(())
}

#[test]
fn test_class_tokens_lexer() -> Result<()> {
    assert_eq!(
        lex_all(b"%a%d%l%s%u%w%x%p%c%g")?,
        vec![
            Token::Class(b'a'),
            Token::Class(b'd'),
            Token::Class(b'l'),
            Token::Class(b's'),
            Token::Class(b'u'),
            Token::Class(b'w'),
            Token::Class(b'x'),
            Token::Class(b'p'),
            Token::Class(b'c'),
            Token::Class(b'g')
        ]
    );
    assert_eq!(
        lex_all(b"%A%D%L%S%U%W%X%P%C%G")?,
        vec![
            Token::Class(b'A'),
            Token::Class(b'D'),
            Token::Class(b'L'),
            Token::Class(b'S'),
            Token::Class(b'U'),
            Token::Class(b'W'),
            Token::Class(b'X'),
            Token::Class(b'P'),
            Token::Class(b'C'),
            Token::Class(b'G')
        ]
    );
    Ok(())
}

#[test]
fn test_special_escape_tokens_lexer() -> Result<()> {
    assert_eq!(
        lex_all(b"%b()%f")?,
        vec![Token::Balanced(b'(', b')'), Token::Frontier]
    );
    Ok(())
}

#[test]
fn test_capture_ref_tokens_lexer() -> Result<()> {
    assert_eq!(
        lex_all(b"%1%2%9")?,
        vec![
            Token::CaptureRef(1),
            Token::CaptureRef(2),
            Token::CaptureRef(9)
        ]
    );
    Ok(())
}

#[test]
fn test_mixed_tokens_lexer() -> Result<()> {
    assert_eq!(
        lex_all(b"(a%d+)%1?")?,
        vec![
            Token::LParen,
            Token::Literal(b'a'),
            Token::Class(b'd'),
            Token::Plus,
            Token::RParen,
            Token::CaptureRef(1),
            Token::Question
        ]
    );
    Ok(())
}

#[test]
fn test_lexer_throw_errors() {
    assert!(matches!(lex_all(b"%"), Err(Error::Lexer(_))));
    assert!(matches!(lex_all(b"%q"), Err(Error::Lexer(_))));
    assert!(matches!(lex_all(b"abc%"), Err(Error::Lexer(_))));
}

#[test]
fn test_quantifiers_lexer() {
    assert_eq!(
        lex_all(b"%d+").unwrap(),
        vec![Token::Class(b'd'), Token::Plus]
    );
    assert_eq!(lex_all(b"%]").unwrap(), vec![Token::EscapedLiteral(b']')]);
    assert_eq!(lex_all(b"%)").unwrap(), vec![Token::EscapedLiteral(b')')]);
    assert_eq!(lex_all(b"%*").unwrap(), vec![Token::EscapedLiteral(b'*')]);
    assert_eq!(lex_all(b"%+").unwrap(), vec![Token::EscapedLiteral(b'+')]);
    assert_eq!(lex_all(b"%?").unwrap(), vec![Token::EscapedLiteral(b'?')]);
    assert_eq!(lex_all(b"%-").unwrap(), vec![Token::EscapedLiteral(b'-')]);
    assert_eq!(
        lex_all(b"[ab]+").unwrap(),
        vec![
            Token::LBracket,
            Token::Literal(b'a'),
            Token::Literal(b'b'),
            Token::RBracket,
            Token::Plus
        ]
    );
    assert_eq!(
        lex_all(b"[ab]*").unwrap(),
        vec![
            Token::LBracket,
            Token::Literal(b'a'),
            Token::Literal(b'b'),
            Token::RBracket,
            Token::Star
        ]
    );
    assert_eq!(
        lex_all(b"[ab]?").unwrap(),
        vec![
            Token::LBracket,
            Token::Literal(b'a'),
            Token::Literal(b'b'),
            Token::RBracket,
            Token::Question
        ]
    );
    assert_eq!(
        lex_all(b"[ab]-").unwrap(),
        vec![
            Token::LBracket,
            Token::Literal(b'a'),
            Token::Literal(b'b'),
            Token::RBracket,
            Token::Minus
        ]
    );
    assert_eq!(
        lex_all(b"a*").unwrap(),
        vec![Token::Literal(b'a'), Token::Star]
    );
    assert_eq!(
        lex_all(b"a+").unwrap(),
        vec![Token::Literal(b'a'), Token::Plus]
    );
    assert_eq!(
        lex_all(b"a?").unwrap(),
        vec![Token::Literal(b'a'), Token::Question]
    );
    assert_eq!(
        lex_all(b"a-").unwrap(),
        vec![Token::Literal(b'a'), Token::Minus]
    );
    assert_eq!(
        lex_all(b"(abc)+").unwrap(),
        vec![
            Token::LParen,
            Token::Literal(b'a'),
            Token::Literal(b'b'),
            Token::Literal(b'c'),
            Token::RParen,
            Token::Plus
        ]
    );
    assert_eq!(
        lex_all(b"(abc)*").unwrap(),
        vec![
            Token::LParen,
            Token::Literal(b'a'),
            Token::Literal(b'b'),
            Token::Literal(b'c'),
            Token::RParen,
            Token::Star
        ]
    );
    assert_eq!(
        lex_all(b"(abc)?").unwrap(),
        vec![
            Token::LParen,
            Token::Literal(b'a'),
            Token::Literal(b'b'),
            Token::Literal(b'c'),
            Token::RParen,
            Token::Question
        ]
    );
    assert_eq!(
        lex_all(b"(abc)-").unwrap(),
        vec![
            Token::LParen,
            Token::Literal(b'a'),
            Token::Literal(b'b'),
            Token::Literal(b'c'),
            Token::RParen,
            Token::Minus
        ]
    );
    assert_eq!(
        lex_all(b"(%d%d%d%d)-").unwrap(),
        vec![
            Token::LParen,
            Token::Class(b'd'),
            Token::Class(b'd'),
            Token::Class(b'd'),
            Token::Class(b'd'),
            Token::RParen,
            Token::Minus
        ]
    );
    assert_eq!(
        lex_all(b"[*]").unwrap(),
        vec![Token::LBracket, Token::Literal(b'*'), Token::RBracket]
    );
    assert_eq!(
        lex_all(b"[+]").unwrap(),
        vec![Token::LBracket, Token::Literal(b'+'), Token::RBracket]
    );
    assert_eq!(
        lex_all(b"[?]").unwrap(),
        vec![Token::LBracket, Token::Literal(b'?'), Token::RBracket]
    );
    assert_eq!(
        lex_all(b"[-]").unwrap(),
        vec![Token::LBracket, Token::Literal(b'-'), Token::RBracket]
    );
    assert_eq!(
        lex_all(b"[a-z]").unwrap(),
        vec![
            Token::LBracket,
            Token::Literal(b'a'),
            Token::Literal(b'-'),
            Token::Literal(b'z'),
            Token::RBracket
        ]
    );
    assert_eq!(
        lex_all(b"[%]]").unwrap(),
        vec![
            Token::LBracket,
            Token::EscapedLiteral(b']'),
            Token::RBracket
        ]
    );
    assert_eq!(
        lex_all(b"[%-]").unwrap(),
        vec![
            Token::LBracket,
            Token::EscapedLiteral(b'-'),
            Token::RBracket
        ]
    );
    assert_eq!(
        lex_all(b"[%%]").unwrap(),
        vec![
            Token::LBracket,
            Token::EscapedLiteral(b'%'),
            Token::RBracket
        ]
    )
}
