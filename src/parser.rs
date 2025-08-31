use super::{
    Error, LUA_MAXCAPTURES, Result,
    ast::{AstNode, AstRoot, Quantifier},
    charset::CharSet,
    lexer::{Lexer, Token},
};
use std::iter::Peekable;

const fn token_to_byte(token: &Token) -> u8 {
    match token {
        Token::Literal(b) | Token::EscapedLiteral(b) => *b,
        Token::Any => b'.',
        Token::LParen => b'(',
        Token::RParen => b')',
        Token::LBracket => b'[',
        Token::RBracket => b']',
        Token::Caret => b'^',
        Token::Dollar => b'$',
        Token::Star => b'*',
        Token::Plus => b'+',
        Token::Question => b'?',
        Token::Minus => b'-',
        Token::Percent => b'%',
        Token::Class(c) => *c,
        Token::Balanced(_, _) => b'b',
        Token::Frontier => b'f',
        Token::CaptureRef(d) => b'0' + *d,
    }
}

pub struct Parser {
    tokens: Peekable<std::vec::IntoIter<Token>>,
    capture_count: usize,
}

impl Parser {
    pub fn new(pattern: &[u8]) -> Result<Self> {
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

    pub fn parse(&mut self) -> Result<AstRoot> {
        let ast = self.parse_sequence(None)?;

        if let Some(token) = self.tokens.peek() {
            return Err(Error::Parser(format!(
                "malformed pattern (unexpected token {token:?} after end of pattern)"
            )));
        }

        if self.capture_count > LUA_MAXCAPTURES {
            return Err(Error::Parser(format!(
                "pattern has too many captures (limit is {LUA_MAXCAPTURES})"
            )));
        }

        Ok(AstRoot::new(ast, self.capture_count))
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
            Token::Literal(b')') => Err(Error::Parser(
                "malformed pattern (unexpected ')')".to_string(),
            )),
            Token::Literal(b']') => Err(Error::Parser(
                "malformed pattern (unexpected ']')".to_string(),
            )),
            Token::Literal(b) | Token::EscapedLiteral(b) => Ok(AstNode::Literal(b)),
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
                token_to_byte(&token)
            ))),
            Token::Minus => Ok(AstNode::Literal(b'-')),
            Token::Percent => Err(Error::Parser(
                "internal error: Percent token should not reach parser base".to_string(),
            )),
            Token::CaptureRef(n) => Ok(AstNode::CaptureRef(n as usize)),
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

                    if self.tokens.peek() == Some(&Token::Literal(b'-')) {
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
                    let token = self.tokens.next().unwrap(); // TODO: remove `unwrap`
                    let byte = token_to_byte(&token);
                    set.add_byte(byte);
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
                "pattern has too many captures (limit is {LUA_MAXCAPTURES})"
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
