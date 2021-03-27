use crate::ast::expr::compound::Arg;
use crate::ast::expr::range::Range;
use crate::ast::expr::Expr;
use crate::ast::expr::PlaceExpr;

#[derive(Debug, PartialEq, Clone)]
pub struct Binary<'a> {
    pub left: Box<Expr<'a>>,
    pub right: Box<Expr<'a>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Assign<'a> {
    pub place: Box<PlaceExpr<'a>>,
    pub expr: Box<Expr<'a>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Property<'a> {
    pub expr: Box<Expr<'a>>,
    pub name: &'a str,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Index<'a> {
    pub expr: Box<Expr<'a>>,
    pub index: Box<Expr<'a>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Slice<'a> {
    pub expr: Box<Expr<'a>>,
    pub range: Range<'a>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Call<'a> {
    pub expr: Box<Expr<'a>>,
    pub args: Arg<'a>,
}
