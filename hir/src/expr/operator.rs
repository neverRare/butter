use crate::expr::range::Range;
use crate::expr::Expr;
use crate::expr::PlaceExpr;
use crate::expr::Record;

#[derive(Debug, PartialEq, Clone)]
pub struct Assign<'a, T> {
    pub place: Box<PlaceExpr<'a, T>>,
    pub expr: Box<Expr<'a, T>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Property<'a, T> {
    pub expr: Box<Expr<'a, T>>,
    pub name: &'a str,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Slice<'a, T> {
    pub expr: Box<Expr<'a, T>>,
    pub range: Range<'a, T>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct NamedArgCall<'a, T> {
    pub expr: Box<Expr<'a, T>>,
    pub args: Record<'a, T>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct UnnamedArgCall<'a, T> {
    pub expr: Box<Expr<'a, T>>,
    pub args: Box<[Expr<'a, T>]>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Tag<'a, T> {
    pub tag: &'a str,
    pub expr: Option<Box<Expr<'a, T>>>,
}
