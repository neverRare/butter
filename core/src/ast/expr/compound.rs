use crate::ast::expr::Expr;

pub struct Array<'a> {
    pub elements: Vec<Element<'a>>,
}
pub enum Element<'a> {
    Element(Expr<'a>),
    Splat(Expr<'a>),
}
pub struct Struct<'a> {
    pub splats: Vec<Expr<'a>>,
    pub fields: Vec<Field<'a>>,
}
pub struct Field<'a> {
    pub name: &'a str,
    pub expr: Expr<'a>,
}
pub enum Arg<'a> {
    Named(Struct<'a>),
    Unnamed(Vec<Expr<'a>>),
}
