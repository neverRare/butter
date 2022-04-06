use crate::{
    expr::{Expr, Fun},
    pattern::Pattern,
    Atom,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<T> {
    Declare(Declare<T>),
    FunDeclare(FunDeclare<T>),
    Expr(Expr<T>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Declare<T> {
    pub pattern: Pattern<T>,
    pub expr: Expr<T>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct FunDeclare<T> {
    pub ident: Atom,
    pub fun: Fun<T>,
    pub ty: T,
}
