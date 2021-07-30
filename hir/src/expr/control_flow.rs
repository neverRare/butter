use crate::expr::Expr;
use crate::pattern::Pattern;
use crate::statement::Statement;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum ControlFlow<'a, T> {
    Block(Block<'a, T>),
    If(If<'a, T>),
    For(For<'a, T>),
    While(While<'a, T>),
    Loop(Block<'a, T>),
    Match(Match<'a, T>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Block<'a, T> {
    pub statement: Box<[Statement<'a, T>]>,
    pub expr: Option<Box<Expr<'a, T>>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct If<'a, T> {
    pub condition: Box<Expr<'a, T>>,
    pub body: Block<'a, T>,
    pub else_part: Option<Box<ControlFlow<'a, T>>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct For<'a, T> {
    pub pattern: Pattern<'a, T>,
    pub expr: Box<Expr<'a, T>>,
    pub body: Block<'a, T>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct While<'a, T> {
    pub condition: Box<Expr<'a, T>>,
    pub body: Block<'a, T>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Match<'a, T> {
    pub expr: Box<Expr<'a, T>>,
    pub arm: Box<[MatchArm<'a, T>]>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct MatchArm<'a, T> {
    pub pattern: Pattern<'a, T>,
    pub expr: Box<Expr<'a, T>>,
}
