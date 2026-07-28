//! Abstract syntax tree for the Rob language.
//!
//! Bootstrap subset: source spans, literals, and arithmetic expressions. It will
//! grow to cover the full grammar as the parser does.

/// A half-open byte range `[start, end)` into the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Byte offset of the first character.
    pub start: usize,
    /// Byte offset one past the last character.
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
    /// An integer literal.
    Int(i64),
    /// A floating-point literal.
    Float(f64),
}

/// A binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    /// Addition, `+`.
    Add,
    /// Subtraction, `-`.
    Sub,
    /// Multiplication, `*`.
    Mul,
    /// Division, `/`.
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
    /// A literal constant.
    Literal(Literal),
    /// A bare identifier.
    Ident(String),
    /// A binary operation.
    Binary {
        /// The operator.
        op: BinOp,
        /// The left operand.
        left: Box<Expr>,
        /// The right operand.
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
