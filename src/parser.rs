use super::{
    Error, LUA_MAXCAPTURES, Result,
    ast::{AstNode, AstRoot, Quantifier},
    charset::CharSet,
    lexer::{Lexer, PosToken, Token},
};

/// Parses a Lua [pattern string](https://www.lua.org/manual/5.3/manual.html#6.4.1) into an AST.
///
/// # Errors
///
/// If the pattern string cannot be parsed, an [`Error`] is returned.
pub fn parse_pattern(pattern: &[u8]) -> Result<AstRoot> {
    Parser::new(pattern)?.parse()
}

/// Converts a pattern string into an AST.
struct Parser<'a> {
    lexer: Lexer<'a>,
    capture_count: usize,
}

impl<'a> Parser<'a> {
    /// Creates a new parser for the given `pattern`.
    ///
    /// # Errors
    ///
    /// If the first byte sequence in the input is not a valid token, an
    /// [`Error`] is returned.
    pub fn new(pattern: &'a [u8]) -> Result<Self> {
        Ok(Parser {
            lexer: Lexer::new(pattern)?,
            capture_count: 0,
        })
    }

    /// Converts the pattern into an [`AstRoot`], consuming the parser.
    ///
    /// # Errors
    ///
    /// If the pattern string is invalid, an [`Error`] is returned.
    pub fn parse(mut self) -> Result<AstRoot> {
        let ast = self.parse_sequence(None)?;

        if let Some(PosToken { pos, token }) = self.lexer.peek() {
            return Err(Error::UnexpectedToken {
                pos,
                lit: token.to_byte(),
            });
        }

        if self.capture_count > LUA_MAXCAPTURES {
            return Err(Error::Captures(self.capture_count));
        }

        Ok(AstRoot::new(ast, self.capture_count))
    }

    fn parse_sequence(&mut self, end_token: Option<Token>) -> Result<Vec<AstNode>> {
        let mut ast = Vec::new();

        while let Some(PosToken { token, .. }) = self.lexer.peek()
            && Some(token) != end_token
        {
            ast.push(self.parse_item()?);
        }

        Ok(ast)
    }

    fn parse_item(&mut self) -> Result<AstNode> {
        let mut base_item = self.parse_base()?;

        let quantifier = match self.lexer.peek().map(|t| t.token) {
            Some(Token::Star) => Some(Quantifier::Star),
            Some(Token::Plus) => Some(Quantifier::Plus),
            Some(Token::Question) => Some(Quantifier::Question),
            Some(Token::Minus) => Some(Quantifier::Minus),
            _ => None,
        };

        if let Some(quantifier) = quantifier {
            let PosToken { pos, token } = self.lexer.next().unwrap().unwrap();

            match base_item {
                AstNode::AnchorStart | AstNode::AnchorEnd | AstNode::Frontier(_) => {
                    return Err(Error::UnexpectedToken {
                        pos,
                        lit: token.to_byte(),
                    })?;
                }
                _ => {}
            }

            base_item = AstNode::Quantified {
                item: Box::new(base_item),
                quantifier,
            };
        }

        Ok(base_item)
    }

    fn parse_base(&mut self) -> Result<AstNode> {
        let Some(PosToken { pos, token }) = self.lexer.next()? else {
            return Err(Error::UnexpectedEndOfPattern {
                pos: self.lexer.tell(),
            });
        };

        match token {
            Token::Literal(b')' | b']')
            | Token::RParen
            | Token::RBracket
            | Token::Star
            | Token::Plus
            | Token::Question => Err(Error::UnexpectedToken {
                pos,
                lit: token.to_byte(),
            }),
            Token::Percent => Err(Error::InternalError { pos }),

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
                self.lexer.expect(Token::LBracket)?;
                let set_node = self.parse_set()?;
                if let AstNode::Set(charset) = set_node {
                    Ok(AstNode::Frontier(charset))
                } else {
                    unreachable!("parse_set should return AstNode::Set");
                }
            }
            Token::Minus => Ok(AstNode::Literal(b'-')),
            Token::CaptureRef(n) => Ok(AstNode::CaptureRef(n as usize)),
        }
    }

    fn parse_set(&mut self) -> Result<AstNode> {
        let mut set = CharSet::new();
        let mut negated = false;

        if self.lexer.consume(Token::Caret)? {
            negated = true;
        }

        while let Some(PosToken { pos, token }) = self.lexer.until(Token::RBracket)? {
            match token {
                Token::Class(c) => {
                    set.add_class(c)
                        .map_err(|err| Error::CharSet { pos, err })?;
                }
                Token::Literal(lit) => {
                    if self.lexer.consume(Token::Literal(b'-'))? {
                        match self.lexer.peek().map(|token| *token) {
                            // [a-]
                            Some(Token::RBracket) => {
                                set.add_byte(lit);
                                set.add_byte(b'-');
                            }
                            // [a-z]
                            Some(Token::Literal(range_end)) => {
                                self.lexer.next()?;
                                set.add_range(lit, range_end)
                                    .map_err(|err| Error::CharSet { pos, err })?;
                            }
                            _ => set.add_byte(lit),
                        }
                    } else {
                        set.add_byte(lit);
                    }
                }
                _ => {
                    set.add_byte(token.to_byte());
                }
            }
        }

        self.lexer.expect(Token::RBracket)?;

        if self.lexer.consume(Token::RBracket)? {
            set.add_byte(b']');
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
            return Err(Error::Captures(index));
        }

        let inner = self.parse_sequence(Some(Token::RParen))?;

        self.lexer.expect(Token::RParen)?;

        Ok(AstNode::Capture { index, inner })
    }
}
