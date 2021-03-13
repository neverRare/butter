use crate::ast::expr::control_flow::Fun;
use crate::ast::expr::Expr;
use crate::ast::expr::ExprType;
use crate::ast::pattern::Pattern;

#[derive(Debug, PartialEq, Clone)]
pub enum StatementType<'a> {
    Declare(Declare<'a>),
    FunDeclare(FunDeclare<'a>),
    Expr(ExprType<'a>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Statement<'a> {
    pub span: &'a str,
    pub statement: StatementType<'a>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Declare<'a> {
    pub unpack: Pattern<'a>,
    pub expr: Box<Expr<'a>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct FunDeclare<'a> {
    pub ident: &'a str,
    pub fun: Fun<'a>,
}
