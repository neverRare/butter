use crate::expr::compound::Element;
use crate::expr::compound::Record;
use crate::expr::control_flow::ControlFlow;
use crate::expr::operator::Assign;
use crate::expr::operator::NamedArgCall;
use crate::expr::operator::Property;
use crate::expr::operator::Slice;
use crate::expr::operator::Tag;
use crate::expr::operator::UnnamedArgCall;
use crate::expr::range::Range;
use crate::pattern::Pattern;
use std::collections::HashMap;

pub mod compound;
pub mod control_flow;
pub mod operator;
pub mod range;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Literal {
    True,
    False,
    Void,

    UInt(u64),
    Float(f64),
}
// TODO: consider replacing place expressions with PlaceExpr
#[derive(Debug, PartialEq, Clone)]
pub enum Expr<'a, T> {
    Literal(Literal),

    Var(&'a str),

    Minus(Box<Expr<'a, T>>),
    Ref(Box<Expr<'a, T>>),
    MutRef(Box<Expr<'a, T>>),
    Not(Box<Expr<'a, T>>),
    Tag(Tag<'a, T>),

    Assign(Assign<'a, T>),
    ParallelAssign(Box<[Assign<'a, T>]>),

    Array(Box<[Element<'a, T>]>),
    ArrayRange(Range<'a, T>),
    Record(Record<'a, T>),

    Binary(Binary<'a, T>),

    Property(Property<'a, T>),
    Index(Index<'a, T>),
    Slice(Slice<'a, T>),
    NamedArgCall(NamedArgCall<'a, T>),
    UnnamedArgCall(UnnamedArgCall<'a, T>),
    Deref(Box<Expr<'a, T>>),
    Len(Box<Expr<'a, T>>),

    ControlFlow(ControlFlow<'a, T>),
    Fun(Fun<'a, T>),
    Jump(Jump<'a, T>),
}
#[derive(Debug, PartialEq, Clone)]
pub enum PlaceExpr<'a, T> {
    Var(&'a str),
    Property(Property<'a, T>),
    Index(Index<'a, T>),
    Deref(Box<Expr<'a, T>>),
}
impl<'a, T> PlaceExpr<'a, T> {
    pub fn from_expr(expr: Expr<'a, T>) -> Option<Self> {
        Some(match expr {
            Expr::Var(var) => Self::Var(var),
            Expr::Property(prop_expr) => Self::Property(prop_expr),
            Expr::Index(ind_expr) => Self::Index(ind_expr),
            Expr::Deref(expr) => Self::Deref(expr),
            _ => return None,
        })
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Fun<'a, T> {
    pub param: HashMap<&'a str, Pattern<'a, T>>,
    pub body: Box<Expr<'a, T>>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Jump<'a, T> {
    Break(Option<Box<Expr<'a, T>>>),
    Continue,
    Return(Option<Box<Expr<'a, T>>>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Binary<'a, T> {
    pub kind: BinaryType,
    pub left: Box<Expr<'a, T>>,
    pub right: Box<Expr<'a, T>>,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinaryType {
    Add,
    Sub,
    Multiply,
    Div,
    FloorDiv,
    Mod,
    And,
    Or,
    LazyAnd,
    LazyOr,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Concatenate,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Index<'a, T> {
    pub expr: Box<Expr<'a, T>>,
    pub index: Box<Expr<'a, T>>,
}
