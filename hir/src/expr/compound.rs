use crate::expr::Expr;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Element<'a, T> {
    Element(Expr<'a, T>),
    Splat(Expr<'a, T>),
}
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Record<'a, T> {
    pub splats: Box<[Expr<'a, T>]>,
    pub fields: HashMap<&'a str, Expr<'a, T>>,
}
