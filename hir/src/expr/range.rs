use crate::expr::Expr;

#[derive(Debug, PartialEq, Clone)]
pub enum Bound<'a, T> {
    NoBound,
    Inclusive(Box<Expr<'a, T>>),
    Exclusive(Box<Expr<'a, T>>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Range<'a, T> {
    pub left: Bound<'a, T>,
    pub right: Bound<'a, T>,
}
