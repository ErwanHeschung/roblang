//! Tokenizer for the Rob language.
//!
//! Bootstrap subset: integers, identifiers, the four arithmetic operators, and
//! parentheses, built on [`logos`]. The full token set follows in later work.

use logos::Logos;

/// A lexical token.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n]+")]
pub enum Token {
    /// An integer literal, e.g. `42` or `1_000`.
    #[regex(r"[0-9][0-9_]*", |lex| lex.slice().replace('_', "").parse::<i64>().ok())]
    Int(i64),

    /// An identifier, e.g. `foo_bar`.
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", |lex| lex.slice().to_owned())]
    Ident(String),

    /// The `+` operator.
    #[token("+")]
    Plus,

    /// The `-` operator.
    #[token("-")]
    Minus,

    /// The `*` operator.
    #[token("*")]
    Star,

    /// The `/` operator.
    #[token("/")]
    Slash,

    /// A left parenthesis, `(`.
    #[token("(")]
    LParen,

    /// A right parenthesis, `)`.
    #[token(")")]
    RParen,
}

/// The byte range of input that could not be tokenized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    /// The offending byte range.
    pub span: std::ops::Range<usize>,
}

/// Tokenizes `source`, or returns the span of the first untokenizable input.
pub fn tokenize(source: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    for (result, span) in Token::lexer(source).spanned() {
        match result {
            Ok(token) => tokens.push(token),
            Err(()) => return Err(LexError { span }),
        }
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_arithmetic() {
        assert_eq!(tokenize("1+2").unwrap(), vec![Token::Int(1), Token::Plus, Token::Int(2)]);
    }

    #[test]
    fn tokenizes_identifier_with_underscores() {
        assert_eq!(tokenize("foo_bar").unwrap(), vec![Token::Ident("foo_bar".to_string())]);
    }

    #[test]
    fn rejects_unknown_character() {
        assert!(tokenize("1 @ 2").is_err());
    }
}
