use crate::ast::expr::Expr;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Element<'a> {
    Element(Expr<'a>),
    Splat(Expr<'a>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Struct<'a> {
    pub splats: Vec<Expr<'a>>,
    pub fields: HashMap<&'a str, Expr<'a>>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Arg<'a> {
    Named(Struct<'a>),
    Unnamed(Vec<Expr<'a>>),
}
