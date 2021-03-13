use crate::ast::expr::Expr;

#[derive(Debug, PartialEq, Clone)]
pub struct Array<'a> {
    pub elements: Vec<Element<'a>>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Element<'a> {
    Element(Expr<'a>),
    Splat(Expr<'a>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Struct<'a> {
    pub splats: Vec<Expr<'a>>,
    pub fields: Vec<Field<'a>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Field<'a> {
    pub name: &'a str,
    pub expr: Expr<'a>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Arg<'a> {
    Named(Struct<'a>),
    Unnamed(Vec<Expr<'a>>),
}
