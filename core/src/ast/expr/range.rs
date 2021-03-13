use crate::ast::expr::Expr;

#[derive(Debug, PartialEq, Clone)]
pub enum Bound<'a> {
    NoBound,
    Inclusive(Box<Expr<'a>>),
    Exclusive(Box<Expr<'a>>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Range<'a> {
    left: Bound<'a>,
    right: Bound<'a>,
}
