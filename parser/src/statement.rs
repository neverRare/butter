use crate::expr::control_flow::Fun;
use crate::expr::Expr;
use crate::pattern::Pattern;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<'a> {
    Declare(Declare<'a>),
    FunDeclare(FunDeclare<'a>),
    Expr(Expr<'a>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Declare<'a> {
    pub pattern: Pattern<'a>,
    pub expr: Box<Expr<'a>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct FunDeclare<'a> {
    pub ident: &'a str,
    pub fun: Fun<'a>,
}
