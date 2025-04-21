use super::{Error, Result};

pub mod token;

pub use token::Token;

fn is_class_char(c: u8) -> bool {
    matches!(
        c as char,
        'a' | 'c'
            | 'd'
            | 'g'
            | 'l'
            | 'p'
            | 's'
            | 'u'
            | 'w'
            | 'x'
            | 'A'
            | 'C'
            | 'D'
            | 'G'
            | 'L'
            | 'P'
            | 'S'
            | 'U'
            | 'W'
            | 'X'
    )
}

fn is_escapable_magic_char(c: u8) -> bool {
    matches!(
        c,
        b'(' | b')' | b'.' | b'%' | b'[' | b']' | b'*' | b'+' | b'-' | b'?' | b'^' | b'$'
    )
}

pub struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
    in_set: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.as_bytes(),
            pos: 0,
            in_set: false,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let byte = self.peek();
        if byte.is_some() {
            self.pos += 1
        }
        byte
    }

    pub fn next_token(&mut self) -> Result<Option<Token>> {
        let Some(byte) = self.advance() else {
            return Ok(None);
        };

        match byte {
            b'(' => Ok(Some(Token::LParen)),
            b')' => Ok(Some(Token::RParen)),
            b'.' => Ok(Some(Token::Any)),
            b'[' => {
                self.in_set = true;
                Ok(Some(Token::LBracket))
            }
            b']' => {
                if self.in_set {
                    self.in_set = false;
                    Ok(Some(Token::RBracket))
                } else {
                    Ok(Some(Token::Literal(byte)))
                }
            }
            b'^' => Ok(Some(Token::Caret)),
            b'$' => Ok(Some(Token::Dollar)),
            b'*' => Ok(Some(Token::Star)),
            b'+' => Ok(Some(Token::Plus)),
            b'?' => Ok(Some(Token::Question)),
            b'-' => Ok(Some(Token::Minus)),
            b'%' => {
                if self.in_set {
                    if let Some(next_byte_peek) = self.peek() {
                        if is_class_char(next_byte_peek) {
                            self.advance();
                            Ok(Some(Token::Class(next_byte_peek as char)))
                        } else {
                            Ok(Some(Token::Literal(b'%')))
                        }
                    } else {
                        Err(Error::Lexer(
                            "malformed pattern (ends with '%' inside set)".to_string(),
                        ))
                    }
                } else {
                    let Some(next_byte) = self.advance() else {
                        return Err(Error::Lexer(
                            "malformed pattern (ends with '%')".to_string(),
                        ));
                    };
                    match next_byte {
                        c if is_escapable_magic_char(c) => Ok(Some(Token::Literal(c))),
                        c if is_class_char(c) => Ok(Some(Token::Class(c as char))),
                        b'b' => {
                            let Some(d1) = self.advance() else {
                                return Err(Error::Lexer(
                                    "malformed pattern (%b needs two characters)".to_string(),
                                ));
                            };
                            let Some(d2) = self.advance() else {
                                return Err(Error::Lexer(
                                    "malformed pattern (%b needs two characters)".to_string(),
                                ));
                            };
                            Ok(Some(Token::Balanced(d1, d2)))
                        }
                        b'f' => Ok(Some(Token::Frontier)),
                        d @ b'1'..=b'9' => Ok(Some(Token::CaptureRef(d - b'0'))),
                        _ => Err(Error::Lexer(format!(
                            "malformed pattern (invalid use of '%%' in pattern: %{})",
                            next_byte as char
                        ))),
                    }
                }
            }

            _ => Ok(Some(Token::Literal(byte))),
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(Some(token)) => Some(Ok(token)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_all(input: &str) -> Result<Vec<Token>> {
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
            lex_all("abc")?,
            vec![
                Token::Literal(b'a'),
                Token::Literal(b'b'),
                Token::Literal(b'c')
            ]
        );
        assert_eq!(
            lex_all("a.c")?,
            vec![Token::Literal(b'a'), Token::Any, Token::Literal(b'c')]
        );
        assert_eq!(lex_all("()")?, vec![Token::LParen, Token::RParen]);
        assert_eq!(lex_all("[]")?, vec![Token::LBracket, Token::RBracket]);
        assert_eq!(
            lex_all("^$*+?-")?,
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
        assert_eq!(lex_all("%%")?, vec![Token::Literal(b'%')]);
        assert_eq!(
            lex_all("%.%a")?,
            vec![Token::Literal(b'.'), Token::Class('a')]
        );
        assert_eq!(lex_all("%(")?, vec![Token::Literal(b'(')]);
        assert_eq!(lex_all("%)")?, vec![Token::Literal(b')')]);
        assert_eq!(lex_all("%[")?, vec![Token::Literal(b'[')]);
        assert_eq!(lex_all("%]")?, vec![Token::Literal(b']')]);
        assert_eq!(lex_all("%*")?, vec![Token::Literal(b'*')]);
        assert_eq!(lex_all("%+")?, vec![Token::Literal(b'+')]);
        assert_eq!(lex_all("%?")?, vec![Token::Literal(b'?')]);
        assert_eq!(lex_all("%-")?, vec![Token::Literal(b'-')]);
        assert_eq!(lex_all("%^")?, vec![Token::Literal(b'^')]);
        assert_eq!(lex_all("%$")?, vec![Token::Literal(b'$')]);
        Ok(())
    }

    #[test]
    fn test_class_tokens_lexer() -> Result<()> {
        assert_eq!(
            lex_all("%a%d%l%s%u%w%x%p%c%g")?,
            vec![
                Token::Class('a'),
                Token::Class('d'),
                Token::Class('l'),
                Token::Class('s'),
                Token::Class('u'),
                Token::Class('w'),
                Token::Class('x'),
                Token::Class('p'),
                Token::Class('c'),
                Token::Class('g')
            ]
        );
        assert_eq!(
            lex_all("%A%D%L%S%U%W%X%P%C%G")?,
            vec![
                Token::Class('A'),
                Token::Class('D'),
                Token::Class('L'),
                Token::Class('S'),
                Token::Class('U'),
                Token::Class('W'),
                Token::Class('X'),
                Token::Class('P'),
                Token::Class('C'),
                Token::Class('G')
            ]
        );
        Ok(())
    }

    #[test]
    fn test_special_escape_tokens_lexer() -> Result<()> {
        assert_eq!(
            lex_all("%b()%f")?,
            vec![Token::Balanced(b'(', b')'), Token::Frontier]
        );
        Ok(())
    }

    #[test]
    fn test_capture_ref_tokens_lexer() -> Result<()> {
        assert_eq!(
            lex_all("%1%2%9")?,
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
            lex_all("(a%d+)%1?")?,
            vec![
                Token::LParen,
                Token::Literal(b'a'),
                Token::Class('d'),
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
        assert!(matches!(lex_all("%"), Err(Error::Lexer(_))));
        assert!(matches!(lex_all("%q"), Err(Error::Lexer(_))));
        assert!(matches!(lex_all("abc%"), Err(Error::Lexer(_))));
    }
}
