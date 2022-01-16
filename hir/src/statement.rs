use crate::{
    expr::{Expr, Fun},
    pattern::Pattern,
};
use string_cache::DefaultAtom;

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
    pub ident: DefaultAtom,
    pub fun: Fun<T>,
    pub ty: T,
}
