use super::{
    Error, LUA_MAXCAPTURES, Result,
    ast::{AstNode, Quantifier},
    charset::CharSet,
    lexer::{Lexer, Token},
};
use std::iter::Peekable;

const fn token_to_byte(token: &Token) -> Option<u8> {
    match token {
        Token::Literal(b) => Some(*b),
        Token::Any => Some(b'.'),
        Token::LParen => Some(b'('),
        Token::RParen => Some(b')'),
        Token::LBracket => Some(b'['),
        Token::RBracket => Some(b']'),
        Token::Caret => Some(b'^'),
        Token::Dollar => Some(b'$'),
        Token::Star => Some(b'*'),
        Token::Plus => Some(b'+'),
        Token::Question => Some(b'?'),
        Token::Minus => Some(b'-'),
        Token::Percent => Some(b'%'),
        Token::Class(c) => Some(*c),
        Token::Balanced(_, _) => Some(b'b'),
        Token::Frontier => Some(b'f'),
        Token::CaptureRef(d) => Some(b'0' + *d),
    }
}

pub struct Parser {
    tokens: Peekable<std::vec::IntoIter<Token>>,
    capture_count: usize,
}

impl Parser {
    pub fn new(pattern: &str) -> Result<Self> {
        let mut lexer = Lexer::new(pattern);
        let mut token_vec = Vec::new();
        loop {
            match lexer.next_token() {
                Ok(Some(token)) => token_vec.push(token),
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(Parser {
            tokens: token_vec.into_iter().peekable(),
            capture_count: 0,
        })
    }

    pub fn parse(&mut self) -> Result<Vec<AstNode>> {
        let ast = self.parse_sequence(None)?;

        if let Some(token) = self.tokens.peek() {
            return Err(Error::Parser(format!(
                "malformed pattern (unexpected token {:?} after end of pattern)",
                token
            )));
        }

        if self.capture_count > LUA_MAXCAPTURES {
            return Err(Error::Parser(format!(
                "pattern has too many captures (limit is {})",
                LUA_MAXCAPTURES
            )));
        }

        Ok(ast)
    }

    fn parse_sequence(&mut self, end_token: Option<&Token>) -> Result<Vec<AstNode>> {
        let mut ast = Vec::new();

        while self.tokens.peek().is_some() && self.tokens.peek() != end_token {
            ast.push(self.parse_item()?);
        }

        if end_token.is_some() && self.tokens.peek() != end_token {
            return Err(Error::Parser(format!(
                "malformed pattern (unexpected end, expected {:?})",
                end_token.unwrap()
            )));
        }

        Ok(ast)
    }

    fn parse_item(&mut self) -> Result<AstNode> {
        let mut base_item = self.parse_base()?;

        let quantifier = match self.tokens.peek() {
            Some(Token::Star) => Some(Quantifier::Star),
            Some(Token::Plus) => Some(Quantifier::Plus),
            Some(Token::Question) => Some(Quantifier::Question),
            Some(Token::Minus) => Some(Quantifier::Minus),
            _ => None,
        };

        if let Some(q) = quantifier {
            self.tokens.next();

            match base_item {
                AstNode::AnchorStart | AstNode::AnchorEnd | AstNode::Frontier(_) => {
                    return Err(Error::Parser(
                        "pattern item cannot be quantified".to_string(),
                    ));
                }
                _ => {}
            }

            base_item = AstNode::Quantified {
                item: Box::new(base_item),
                quantifier: q,
            };
        }

        Ok(base_item)
    }

    fn parse_base(&mut self) -> Result<AstNode> {
        let Some(token) = self.tokens.next() else {
            return Err(Error::Parser("unexpected end of pattern".to_string()));
        };

        match token {
            Token::Literal(b) => Ok(AstNode::Literal(b)),
            Token::Any => Ok(AstNode::Any),
            Token::Caret => Ok(AstNode::AnchorStart),
            Token::Dollar => Ok(AstNode::AnchorEnd),

            Token::Class(c) => {
                let negated = c.is_ascii_uppercase();
                let base_byte = if negated { c.to_ascii_lowercase() } else { c };
                if [b'a', b'c', b'd', b'g', b'l', b'p', b's', b'u', b'w', b'x'].contains(&base_byte)
                {
                    Ok(AstNode::Class(base_byte, negated))
                } else {
                    Ok(AstNode::Literal(c))
                }
            }

            Token::LBracket => self.parse_set(),

            Token::LParen => self.parse_capture(),

            Token::Balanced(d1, d2) => Ok(AstNode::Balanced(d1, d2)),
            Token::Frontier => {
                if self.tokens.peek() != Some(&Token::LBracket) {
                    return Err(Error::Parser(
                        "malformed pattern (missing '[' after %f)".to_string(),
                    ));
                }
                self.tokens.next();
                let set_node = self.parse_set()?;
                if let AstNode::Set(charset) = set_node {
                    Ok(AstNode::Frontier(charset))
                } else {
                    unreachable!("parse_set should return AstNode::Set");
                }
            }

            Token::RParen => Err(Error::Parser(
                "invalid pattern (unexpected ')')".to_string(),
            )),
            Token::RBracket => Err(Error::Parser(
                "invalid pattern (unexpected ']')".to_string(),
            )),
            Token::Star | Token::Plus | Token::Question => Err(Error::Parser(format!(
                "invalid pattern (quantifier '{}' must follow an item)",
                token_to_byte(&token).unwrap_or(b'?')
            ))),
            Token::Minus => Ok(AstNode::Literal(b'-')),
            Token::Percent => Err(Error::Parser(
                "internal error: Percent token should not reach parser base".to_string(),
            )),
            Token::CaptureRef(_) => Err(Error::Parser(
                "invalid pattern (capture reference %n not allowed here)".to_string(),
            )),
        }
    }

    fn parse_set(&mut self) -> Result<AstNode> {
        let mut set = CharSet::new();
        let mut negated = false;

        if self.tokens.peek() == Some(&Token::Caret) {
            self.tokens.next();
            negated = true;
        }

        if self.tokens.peek() == Some(&Token::RBracket) {
            self.tokens.next();
            if negated {
                set.invert();
            }
            return Ok(AstNode::Set(set));
        }

        if self.tokens.peek() == Some(&Token::RBracket) {
            self.tokens.next();
            set.add_byte(b']');
        }

        while self.tokens.peek().is_some() && self.tokens.peek() != Some(&Token::RBracket) {
            match self.tokens.peek().cloned() {
                Some(Token::Class(c)) => {
                    self.tokens.next();
                    set.add_class(c)?;
                }
                Some(Token::Literal(b)) => {
                    let current_byte = b;
                    self.tokens.next();

                    if self.tokens.peek() == Some(&Token::Minus) {
                        let mut iter_clone = self.tokens.clone();
                        iter_clone.next();

                        if let Some(Token::Literal(next_b)) = iter_clone.peek() {
                            let next_b_val = *next_b;
                            self.tokens.next();
                            self.tokens.next();
                            set.add_range(current_byte, next_b_val)?;
                        } else {
                            set.add_byte(current_byte);
                        }
                    } else {
                        set.add_byte(current_byte);
                    }
                }
                Some(Token::Minus) => {
                    self.tokens.next();
                    set.add_byte(b'-');
                }
                Some(Token::Percent) => {
                    self.tokens.next();
                    set.add_byte(b'%');
                }
                Some(_) => {
                    let token = self.tokens.next().unwrap();
                    if let Some(byte) = token_to_byte(&token) {
                        set.add_byte(byte);
                    } else {
                        return Err(Error::Parser(format!(
                            "invalid token {:?} inside character set",
                            token
                        )));
                    }
                }
                None => unreachable!(),
            }
        }

        if self.tokens.peek() == Some(&Token::RBracket) {
            self.tokens.next();
        } else {
            return Err(Error::Parser(
                "malformed pattern (unfinished character class)".to_string(),
            ));
        }

        if negated {
            set.invert();
        }

        Ok(AstNode::Set(set))
    }

    fn parse_capture(&mut self) -> Result<AstNode> {
        self.capture_count += 1;
        let index = self.capture_count;
        if index > LUA_MAXCAPTURES {
            return Err(Error::Parser(format!(
                "pattern has too many captures (limit is {})",
                LUA_MAXCAPTURES
            )));
        }

        let inner_ast = self.parse_sequence(Some(&Token::RParen))?;

        if self.tokens.next() != Some(Token::RParen) {
            return Err(Error::Parser(
                "malformed pattern (unclosed capture group)".to_string(),
            ));
        }

        Ok(AstNode::Capture {
            index,
            inner: inner_ast,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::AstNode;
    use crate::charset::CharSet;

    fn parse_ok(pattern: &str) -> Vec<AstNode> {
        Parser::new(pattern)
            .expect("Parser::new failed")
            .parse()
            .expect(&format!("Parser failed for pattern: {}", pattern))
    }

    fn parse_err(pattern: &str) -> Result<Vec<AstNode>> {
        let tokens_vec = Lexer::new(pattern).collect::<Result<Vec<Token>>>()?;
        let mut parser = Parser {
            tokens: tokens_vec.into_iter().peekable(),
            capture_count: 0,
        };
        parser.parse()
    }

    fn quantified(item: AstNode, quantifier: Quantifier) -> AstNode {
        AstNode::Quantified {
            item: Box::new(item),
            quantifier,
        }
    }

    fn make_set(bytes: &[u8], ranges: &[(u8, u8)], classes: &[u8], negated: bool) -> CharSet {
        let mut set = CharSet::new();
        for &b in bytes {
            set.add_byte(b);
        }
        for &(s, e) in ranges {
            set.add_range(s, e).unwrap();
        }
        for &c in classes {
            set.add_class(c).unwrap();
        }
        if negated {
            set.invert();
        }
        set
    }

    #[test]
    fn test_simple_sequence_parser() {
        assert_eq!(
            parse_ok("abc"),
            vec![
                AstNode::Literal(b'a'),
                AstNode::Literal(b'b'),
                AstNode::Literal(b'c')
            ]
        );
        assert_eq!(
            parse_ok("a.c"),
            vec![AstNode::Literal(b'a'), AstNode::Any, AstNode::Literal(b'c')]
        );
        assert_eq!(
            parse_ok("a%dc"),
            vec![
                AstNode::Literal(b'a'),
                AstNode::Class(b'd', false),
                AstNode::Literal(b'c')
            ]
        );
        assert_eq!(
            parse_ok("a%Dc"),
            vec![
                AstNode::Literal(b'a'),
                AstNode::Class(b'd', true),
                AstNode::Literal(b'c')
            ]
        );
    }

    #[test]
    fn test_anchors_parser() {
        assert_eq!(
            parse_ok("^abc$"),
            vec![
                AstNode::AnchorStart,
                AstNode::Literal(b'a'),
                AstNode::Literal(b'b'),
                AstNode::Literal(b'c'),
                AstNode::AnchorEnd
            ]
        );
        assert_eq!(
            parse_ok("abc$"),
            vec![
                AstNode::Literal(b'a'),
                AstNode::Literal(b'b'),
                AstNode::Literal(b'c'),
                AstNode::AnchorEnd
            ]
        );
        assert_eq!(
            parse_ok("^abc"),
            vec![
                AstNode::AnchorStart,
                AstNode::Literal(b'a'),
                AstNode::Literal(b'b'),
                AstNode::Literal(b'c')
            ]
        );
    }

    #[test]
    fn test_quantifiers_parser() {
        assert_eq!(
            parse_ok("a*"),
            vec![quantified(AstNode::Literal(b'a'), Quantifier::Star)]
        );
        assert_eq!(
            parse_ok("a+"),
            vec![quantified(AstNode::Literal(b'a'), Quantifier::Plus)]
        );
        assert_eq!(
            parse_ok("a?"),
            vec![quantified(AstNode::Literal(b'a'), Quantifier::Question)]
        );
        assert_eq!(
            parse_ok("a-"),
            vec![quantified(AstNode::Literal(b'a'), Quantifier::Minus)]
        );
        assert_eq!(
            parse_ok("a.*c+d?e-"),
            vec![
                AstNode::Literal(b'a'),
                quantified(AstNode::Any, Quantifier::Star),
                quantified(AstNode::Literal(b'c'), Quantifier::Plus),
                quantified(AstNode::Literal(b'd'), Quantifier::Question),
                quantified(AstNode::Literal(b'e'), Quantifier::Minus),
            ]
        );
        assert_eq!(
            parse_ok("%d+"),
            vec![quantified(AstNode::Class(b'd', false), Quantifier::Plus)]
        );
        assert_eq!(
            parse_ok(".*"),
            vec![quantified(AstNode::Any, Quantifier::Star)]
        );
    }

    #[test]
    fn test_sets_parser() {
        assert_eq!(
            parse_ok("[]"),
            vec![AstNode::Set(make_set(&[], &[], &[], false))]
        );
        assert_eq!(
            parse_ok("[abc]"),
            vec![AstNode::Set(make_set(&[b'a', b'b', b'c'], &[], &[], false))]
        );
        assert_eq!(
            parse_ok("[^abc]"),
            vec![AstNode::Set(make_set(&[b'a', b'b', b'c'], &[], &[], true))]
        );
        assert_eq!(
            parse_ok("[a-c]"),
            vec![AstNode::Set(make_set(&[], &[(b'a', b'c')], &[], false))]
        );
        assert_eq!(
            parse_ok("[^a-c]"),
            vec![AstNode::Set(make_set(&[], &[(b'a', b'c')], &[], true))]
        );
        assert_eq!(
            parse_ok("[a-c%d]"),
            vec![AstNode::Set(make_set(&[], &[(b'a', b'c')], &[b'd'], false))]
        );
        assert_eq!(
            parse_ok("[a.^$]"),
            vec![AstNode::Set(make_set(
                &[b'a', b'.', b'^', b'$'],
                &[],
                &[],
                false
            ))]
        );
        assert_eq!(
            parse_ok("[%a]"),
            vec![AstNode::Set(make_set(&[], &[], &[b'a'], false))]
        );
        assert_eq!(
            parse_ok("[%]"),
            vec![AstNode::Set(make_set(&[b'%'], &[], &[], false))]
        );
        assert_eq!(
            parse_ok("[-abc]"),
            vec![AstNode::Set(make_set(
                &[b'-', b'a', b'b', b'c'],
                &[],
                &[],
                false
            ))]
        );
        assert_eq!(
            parse_ok("[abc-]"),
            vec![AstNode::Set(make_set(
                &[b'a', b'b', b'c', b'-'],
                &[],
                &[],
                false
            ))]
        );
    }

    #[test]
    fn test_set_quantifier_parser() {
        assert_eq!(
            parse_ok("[abc]*"),
            vec![quantified(
                AstNode::Set(make_set(&[b'a', b'b', b'c'], &[], &[], false)),
                Quantifier::Star
            )]
        );
    }

    #[test]
    fn test_captures_parser() {
        assert_eq!(
            parse_ok("()"),
            vec![AstNode::Capture {
                index: 1,
                inner: vec![]
            }]
        );
        assert_eq!(
            parse_ok("(a)"),
            vec![AstNode::Capture {
                index: 1,
                inner: vec![AstNode::Literal(b'a')]
            }]
        );
        assert_eq!(
            parse_ok("(a%d+)"),
            vec![AstNode::Capture {
                index: 1,
                inner: vec![
                    AstNode::Literal(b'a'),
                    quantified(AstNode::Class(b'd', false), Quantifier::Plus)
                ]
            }]
        );
        assert_eq!(
            parse_ok("(a(b)c)"),
            vec![AstNode::Capture {
                index: 1,
                inner: vec![
                    AstNode::Literal(b'a'),
                    AstNode::Capture {
                        index: 2,
                        inner: vec![AstNode::Literal(b'b')]
                    },
                    AstNode::Literal(b'c')
                ]
            }]
        );
        assert_eq!(
            parse_ok("(a)?"),
            vec![quantified(
                AstNode::Capture {
                    index: 1,
                    inner: vec![AstNode::Literal(b'a')]
                },
                Quantifier::Question
            )]
        );
        assert_eq!(
            parse_ok("a?b"),
            vec![
                quantified(AstNode::Literal(b'a'), Quantifier::Question),
                AstNode::Literal(b'b')
            ]
        );
        assert_eq!(
            parse_ok("a-b"),
            vec![
                quantified(AstNode::Literal(b'a'), Quantifier::Minus),
                AstNode::Literal(b'b')
            ]
        );

        assert_eq!(
            parse_ok("a.*c+d?e-"),
            vec![
                AstNode::Literal(b'a'),
                quantified(AstNode::Any, Quantifier::Star),
                quantified(AstNode::Literal(b'c'), Quantifier::Plus),
                quantified(AstNode::Literal(b'd'), Quantifier::Question),
                quantified(AstNode::Literal(b'e'), Quantifier::Minus),
            ]
        );
        assert_eq!(
            parse_ok("%d+"),
            vec![quantified(AstNode::Class(b'd', false), Quantifier::Plus)]
        );
        assert_eq!(
            parse_ok(".*"),
            vec![quantified(AstNode::Any, Quantifier::Star)]
        );
    }

    #[test]
    fn test_balanced_frontier_parser() {
        assert_eq!(parse_ok("%b()"), vec![AstNode::Balanced(b'(', b')')]);
        assert_eq!(
            parse_ok("%f[ac]"),
            vec![AstNode::Frontier(make_set(&[b'a', b'c'], &[], &[], false))]
        );
    }

    #[test]
    fn test_complex_parser() {
        assert_eq!(
            parse_ok("^(%b())%d*$"),
            vec![
                AstNode::AnchorStart,
                AstNode::Capture {
                    index: 1,
                    inner: vec![AstNode::Balanced(b'(', b')')]
                },
                quantified(AstNode::Class(b'd', false), Quantifier::Star),
                AstNode::AnchorEnd
            ]
        );
    }

    #[test]
    fn test_throw_parser_errors() {
        assert!(
            matches!(parse_err("("), Err(Error::Parser(s)) if s.contains("malformed pattern (unexpected end, expected RParen)"))
        );
        assert!(matches!(parse_err(")"), Err(Error::Parser(s)) if s.contains("unexpected ')'")));
        assert!(
            matches!(parse_err("["), Err(Error::Parser(s)) if s.contains("unfinished character class"))
        );
        assert_eq!(parse_ok("]"), vec![AstNode::Literal(b']')]);
        assert!(
            matches!(parse_err("*"), Err(Error::Parser(s)) if s.contains("must follow an item"))
        );
        assert!(
            matches!(parse_err("^*"), Err(Error::Parser(s)) if s.contains("cannot be quantified"))
        );
        assert!(
            matches!(parse_err("$+"), Err(Error::Parser(s)) if s.contains("cannot be quantified"))
        );
        assert!(
            matches!(parse_err("%b"), Err(Error::Lexer(s)) if s.contains("needs two characters"))
        );
        assert!(
            matches!(parse_err("%bx"), Err(Error::Lexer(s)) if s.contains("needs two characters"))
        );
        assert!(
            matches!(parse_err("%f"), Err(Error::Parser(s)) if s.contains("missing '[' after %f"))
        );
        assert!(
            matches!(parse_err("%fa"), Err(Error::Parser(s)) if s.contains("missing '[' after %f"))
        );
        assert!(
            matches!(parse_err("%f["), Err(Error::Parser(s)) if s.contains("unfinished character class"))
        );
        assert!(
            matches!(parse_err("%f[a"), Err(Error::Parser(s)) if s.contains("unfinished character class"))
        );
        assert!(matches!(parse_err("%z"), Err(Error::Lexer(_))));
        assert!(matches!(parse_err("%1"), Err(Error::Parser(s)) if s.contains("not allowed here")));
        let too_many_captures = "()".repeat(LUA_MAXCAPTURES + 1);
        assert!(
            matches!(parse_err(&too_many_captures), Err(Error::Parser(s)) if s.contains("too many captures"))
        );
    }

    #[test]
    fn test_special_byte_edge_cases_parser() {
        assert_eq!(
            parse_ok("[%%]"),
            vec![AstNode::Set(make_set(&[b'%'], &[], &[], false))]
        );
        assert_eq!(
            parse_ok("[%-]"),
            vec![AstNode::Set(make_set(&[b'%', b'-'], &[], &[], false))]
        );
        assert_eq!(
            parse_ok("[%]]"),
            vec![
                AstNode::Set(make_set(&[b'%'], &[], &[], false)),
                AstNode::Literal(b']')
            ]
        );
        assert_eq!(
            parse_ok("[%[]"),
            vec![AstNode::Set(make_set(&[b'%', b'['], &[], &[], false))]
        );

        assert_eq!(
            parse_ok("%*+%?"),
            vec![
                quantified(AstNode::Literal(b'*'), Quantifier::Plus),
                AstNode::Literal(b'?')
            ]
        );
        assert_eq!(
            parse_ok("[%[]"),
            vec![AstNode::Set(make_set(&[b'%', b'['], &[], &[], false))]
        );
    }

    #[test]
    fn test_nested_complex_patterns_parser() {
        assert_eq!(
            parse_ok("((a+)?(b*))+"),
            vec![quantified(
                AstNode::Capture {
                    index: 1,
                    inner: vec![
                        quantified(
                            AstNode::Capture {
                                index: 2,
                                inner: vec![quantified(AstNode::Literal(b'a'), Quantifier::Plus)]
                            },
                            Quantifier::Question
                        ),
                        AstNode::Capture {
                            index: 3,
                            inner: vec![quantified(AstNode::Literal(b'b'), Quantifier::Star)]
                        }
                    ]
                },
                Quantifier::Plus
            )]
        );

        assert_eq!(
            parse_ok("(%f[%a]%w+)"),
            vec![AstNode::Capture {
                index: 1,
                inner: vec![
                    AstNode::Frontier(make_set(&[], &[], &[b'a'], false)),
                    quantified(AstNode::Class(b'w', false), Quantifier::Plus)
                ]
            }]
        );
    }

    #[test]
    fn test_real_world_patterns_parser() {
        assert!(parse_ok("https?://[%w%.%-%+]+%.%w+").len() > 0);

        assert!(parse_ok("^[%w%.%+%-]+@[%w%.%+%-]+%.%w+$").len() > 0);

        assert!(parse_ok("(%d%d?)/(%d%d?)/(%d%d%d%d)").len() > 0);

        assert!(parse_ok("(%d+)%.(%d+)%.(%d+)%.(%d+)").len() > 0);

        assert!(parse_ok("\"([^\"]+)\":%s*\"([^\"]*)\"").len() > 0);
    }

    #[test]
    fn test_special_lua_pattern_features_parser() {
        assert_eq!(parse_ok("%b<>"), vec![AstNode::Balanced(b'<', b'>')]);
        assert_eq!(parse_ok("%b''"), vec![AstNode::Balanced(b'\'', b'\'')]);
        assert_eq!(parse_ok("%b{}"), vec![AstNode::Balanced(b'{', b'}')]);

        assert_eq!(
            parse_ok("%f[%w]word"),
            vec![
                AstNode::Frontier(make_set(&[], &[], &[b'w'], false)),
                AstNode::Literal(b'w'),
                AstNode::Literal(b'o'),
                AstNode::Literal(b'r'),
                AstNode::Literal(b'd')
            ]
        );

        assert_eq!(
            parse_ok("%f[^%s]%w+"),
            vec![
                AstNode::Frontier(make_set(&[], &[], &[b's'], true)),
                quantified(AstNode::Class(b'w', false), Quantifier::Plus)
            ]
        );
    }
}
