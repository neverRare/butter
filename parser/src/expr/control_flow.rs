use crate::expr::Expr;
use crate::pattern::Pattern;
use crate::statement::Statement;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Break<'a> {
    pub label: Option<&'a str>,
    pub expr: Option<Box<Expr<'a>>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Block<'a> {
    pub statement: Box<[Statement<'a>]>,
    pub expr: Option<Box<Expr<'a>>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Fun<'a> {
    pub param: Param<'a>,
    pub body: Box<Expr<'a>>,
}
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Param<'a> {
    pub order: Box<[&'a str]>,
    pub param: HashMap<&'a str, Pattern<'a>>,
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
#[derive(Debug, PartialEq, Clone)]
pub struct Match<'a> {
    pub expr: Box<Expr<'a>>,
    pub arm: Box<[MatchArm<'a>]>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct MatchArm<'a> {
    pub pattern: Pattern<'a>,
    pub expr: Box<Expr<'a>>,
}
