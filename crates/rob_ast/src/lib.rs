//! Abstract syntax tree for the Rob language.
//!
//! Bootstrap subset: source spans, literals, and arithmetic expressions. It will
//! grow to cover the full grammar as the parser does.

/// A half-open byte range `[start, end)` into the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    /// Creates a span from a start and end byte offset.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// A literal constant.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
}

/// A binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl BinOp {
    /// The source symbol for this operator.
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
        }
    }
}

/// An expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Ident(String),
    /// A binary operation, e.g. `a + b`.
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

impl Expr {
    /// Renders the expression as a fully parenthesized S-expression. Used for
    /// stable snapshot testing of parser output.
    pub fn to_sexpr(&self) -> String {
        match self {
            Expr::Literal(Literal::Int(value)) => value.to_string(),
            Expr::Literal(Literal::Float(value)) => value.to_string(),
            Expr::Ident(name) => name.clone(),
            Expr::Binary { op, left, right } => {
                format!("({} {} {})", op.symbol(), left.to_sexpr(), right.to_sexpr())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sexpr_of_nested_expression() {
        let expr = Expr::Binary {
            op: BinOp::Add,
            left: Box::new(Expr::Literal(Literal::Int(1))),
            right: Box::new(Expr::Binary {
                op: BinOp::Mul,
                left: Box::new(Expr::Literal(Literal::Int(2))),
                right: Box::new(Expr::Ident("x".to_string())),
            }),
        };
        assert_eq!(expr.to_sexpr(), "(+ 1 (* 2 x))");
    }
}
