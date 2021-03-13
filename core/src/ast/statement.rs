use crate::ast::expr::control_flow::Fun;
use crate::ast::expr::Expr;
use crate::ast::expr::ExprType;
use crate::ast::pattern::Pattern;

pub enum StatementType<'a> {
    Declare(Declare<'a>),
    FunDeclare(FunDeclare<'a>),
    Expr(ExprType<'a>),
}
pub struct Statement<'a> {
    pub span: &'a str,
    pub statement: StatementType<'a>,
}
pub struct Declare<'a> {
    pub unpack: Pattern<'a>,
    pub expr: Box<Expr<'a>>,
}
pub struct FunDeclare<'a> {
    pub ident: &'a str,
    pub fun: Fun<'a>,
}
