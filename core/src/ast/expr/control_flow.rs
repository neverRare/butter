use crate::ast::expr::Expr;
use crate::ast::pattern::Param;
use crate::ast::pattern::Pattern;
use crate::ast::statement::Statement;

#[derive(Debug, PartialEq, Clone)]
pub struct Break<'a> {
    pub label: Option<&'a str>,
    pub expr: Option<Box<Expr<'a>>>,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Continue<'a> {
    pub label: Option<&'a str>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Return<'a> {
    pub expr: Option<Box<Expr<'a>>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Block<'a> {
    pub statements: Vec<Statement<'a>>,
    pub expr: Box<Expr<'a>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Fun<'a> {
    pub params: Param<'a>,
    pub body: Box<Expr<'a>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct If<'a> {
    pub condition: Box<Expr<'a>>,
    pub body: Block<'a>,
    pub else_part: Else<'a>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Else<'a> {
    None,
    Else(Block<'a>),
    ElseIf(Box<If<'a>>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct For<'a> {
    pub unpack: Pattern<'a>,
    pub expr: Box<Expr<'a>>,
    pub body: Block<'a>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct While<'a> {
    pub condition: Box<Expr<'a>>,
    pub body: Block<'a>,
}
