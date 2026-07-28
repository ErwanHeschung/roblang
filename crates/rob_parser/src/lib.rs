//! Recursive-descent (Pratt) parser for the Rob language.
//!
//! Bootstrap subset: arithmetic expressions over integers, identifiers, the four
//! binary operators, and parentheses. Produces a [`rob_ast::Expr`].

use rob_ast::{BinOp, Expr, Literal};
use rob_lexer::{Token, tokenize};

/// An error produced while parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The source could not be tokenized.
    Lex,
    /// The input ended before a complete expression was parsed.
    UnexpectedEnd,
    /// An unexpected token was encountered (its debug form is included).
    Unexpected(String),
}

/// Parses `source` as a single expression.
pub fn parse_expr(source: &str) -> Result<Expr, ParseError> {
    let tokens = tokenize(source).map_err(|_| ParseError::Lex)?;
    let mut parser = Parser::new(&tokens);
    let expr = parser.expr(0)?;
    if let Some(token) = parser.peek() {
        return Err(ParseError::Unexpected(format!("{token:?}")));
    }
    Ok(expr)
}

/// Parses `source` and renders the resulting expression as an S-expression.
pub fn parse_to_sexpr(source: &str) -> Result<String, ParseError> {
    Ok(parse_expr(source)?.to_sexpr())
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }

    fn bump(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    /// Pratt expression parser: consume operators whose binding power is at least
    /// `min_bp`.
    fn expr(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let mut left = self.primary()?;
        while let Some(token) = self.peek() {
            let (op, bp) = match token {
                Token::Plus => (BinOp::Add, 1u8),
                Token::Minus => (BinOp::Sub, 1),
                Token::Star => (BinOp::Mul, 2),
                Token::Slash => (BinOp::Div, 2),
                _ => break,
            };
            if bp < min_bp {
                break;
            }
            self.bump();
            let right = self.expr(bp + 1)?;
            left = Expr::Binary { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        match self.bump() {
            Some(Token::Int(value)) => Ok(Expr::Literal(Literal::Int(value))),
            Some(Token::Ident(name)) => Ok(Expr::Ident(name)),
            Some(Token::LParen) => {
                let inner = self.expr(0)?;
                match self.bump() {
                    Some(Token::RParen) => Ok(inner),
                    other => Err(unexpected(other)),
                }
            }
            other => Err(unexpected(other)),
        }
    }
}

fn unexpected(token: Option<Token>) -> ParseError {
    match token {
        Some(token) => ParseError::Unexpected(format!("{token:?}")),
        None => ParseError::UnexpectedEnd,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_precedence() {
        assert_eq!(parse_to_sexpr("1 + 2 * 3").unwrap(), "(+ 1 (* 2 3))");
    }

    #[test]
    fn parses_parentheses() {
        assert_eq!(parse_to_sexpr("(1 + 2) * 3").unwrap(), "(* (+ 1 2) 3)");
    }

    #[test]
    fn rejects_trailing_token() {
        assert!(parse_expr("1 2").is_err());
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(parse_expr(""), Err(ParseError::UnexpectedEnd));
    }
}
