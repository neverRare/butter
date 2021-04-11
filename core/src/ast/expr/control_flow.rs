use crate::ast::expr::Expr;
use crate::ast::pattern::Pattern;
use crate::ast::statement::Statement;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Break<'a> {
    pub label: Option<&'a str>,
    pub expr: Option<Box<Expr<'a>>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Block<'a> {
    pub statement: Vec<Statement<'a>>,
    pub expr: Option<Box<Expr<'a>>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Fun<'a> {
    pub param: HashMap<&'a str, Pattern<'a>>,
    pub body: Box<Expr<'a>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct If<'a> {
    pub condition: Box<Expr<'a>>,
    pub body: Block<'a>,
    pub else_part: Option<Box<Expr<'a>>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct For<'a> {
    pub pattern: Pattern<'a>,
    pub expr: Box<Expr<'a>>,
    pub body: Block<'a>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct While<'a> {
    pub condition: Box<Expr<'a>>,
    pub body: Block<'a>,
}
