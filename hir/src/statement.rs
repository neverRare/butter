use crate::expr::control_flow::Fun;
use crate::expr::Expr;
use crate::pattern::Pattern;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<'a, T> {
    Declare(Declare<'a, T>),
    FunDeclare(FunDeclare<'a, T>),
    Expr(Expr<'a, T>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Declare<'a, T> {
    pub pattern: Pattern<'a, T>,
    pub expr: Expr<'a, T>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct FunDeclare<'a, T> {
    pub ident: &'a str,
    pub fun: Fun<'a, T>,
    pub ty: T,
}
