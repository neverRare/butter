use crate::{
    expr::{Expr, Fun},
    pattern::Pattern,
    pretty_print::{line, PrettyPrint, PrettyPrintTree},
    Atom, PrettyPrintType,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<T> {
    Declare(Declare<T>),
    FunDeclare(FunDeclare<T>),
    Expr(Expr<T>),
}
impl<T: PrettyPrintType> PrettyPrint for Statement<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        match self {
            Statement::Declare(declare) => declare.to_pretty_print(),
            Statement::FunDeclare(_) => todo!(),
            Statement::Expr(expr) => expr.to_pretty_print(),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Declare<T> {
    pub pattern: Pattern<T>,
    pub expr: Expr<T>,
}
impl<T: PrettyPrintType> PrettyPrint for Declare<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        line([
            self.pattern.to_pretty_print(),
            Box::new(" = ".to_string()),
            self.expr.to_pretty_print(),
        ])
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct FunDeclare<T> {
    pub ident: Atom,
    pub fun: Fun<T>,
    pub ty: T,
}
