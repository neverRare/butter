use crate::ast::expr::Expr;

pub enum Bound<'a> {
    NoBound,
    Inclusive(Box<Expr<'a>>),
    Exclusive(Box<Expr<'a>>),
}
pub struct Range<'a> {
    left: Bound<'a>,
    right: Bound<'a>,
}
