use crate::ast::expr::Expr;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Element<'a> {
    Element(Expr<'a>),
    Splat(Expr<'a>),
}
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Struct<'a> {
    pub splats: Vec<Expr<'a>>,
    pub fields: HashMap<&'a str, Expr<'a>>,
}
