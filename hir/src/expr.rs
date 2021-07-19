use crate::expr::compound::Element;
use crate::expr::compound::Record;
use crate::expr::control_flow::Block;
use crate::expr::control_flow::For;
use crate::expr::control_flow::Fun;
use crate::expr::control_flow::If;
use crate::expr::control_flow::Match;
use crate::expr::control_flow::While;
use crate::expr::operator::Assign;
use crate::expr::operator::Binary;
use crate::expr::operator::NamedArgCall;
use crate::expr::operator::Property;
use crate::expr::operator::Slice;
use crate::expr::operator::Tag;
use crate::expr::operator::UnnamedArgCall;
use crate::expr::range::Range;

pub mod compound;
pub mod control_flow;
pub mod operator;
pub mod range;

#[derive(Debug, PartialEq, Clone)]
pub enum Expr<'a, T> {
    True,
    False,
    Void,

    Var(&'a str),
    UInt(u64),
    Float(f64),

    Break(Option<Box<Expr<'a, T>>>),
    Continue,
    Return(Option<Box<Expr<'a, T>>>),

    Minus(Box<Expr<'a, T>>),
    Ref(Box<Expr<'a, T>>),
    MutRef(Box<Expr<'a, T>>),
    Not(Box<Expr<'a, T>>),
    Tag(Tag<'a, T>),

    Add(Binary<'a, T>),
    Sub(Binary<'a, T>),
    Multiply(Binary<'a, T>),
    Div(Binary<'a, T>),
    FloorDiv(Binary<'a, T>),
    Mod(Binary<'a, T>),
    And(Binary<'a, T>),
    Or(Binary<'a, T>),
    LazyAnd(Binary<'a, T>),
    LazyOr(Binary<'a, T>),
    Equal(Binary<'a, T>),
    NotEqual(Binary<'a, T>),
    Greater(Binary<'a, T>),
    GreaterEqual(Binary<'a, T>),
    Less(Binary<'a, T>),
    LessEqual(Binary<'a, T>),
    Concatenate(Binary<'a, T>),

    Assign(Assign<'a, T>),
    ParallelAssign(Box<[Assign<'a, T>]>),

    Array(Box<[Element<'a, T>]>),
    ArrayRange(Range<'a, T>),
    Record(Record<'a, T>),

    Property(Property<'a, T>),
    Index(Binary<'a, T>),
    Slice(Slice<'a, T>),
    NamedArgCall(NamedArgCall<'a, T>),
    UnnamedArgCall(UnnamedArgCall<'a, T>),
    Deref(Box<Expr<'a, T>>),
    Len(Box<Expr<'a, T>>),

    Block(Block<'a, T>),
    Fun(Fun<'a, T>),
    If(If<'a, T>),
    For(For<'a, T>),
    While(While<'a, T>),
    Loop(Block<'a, T>),
    Match(Match<'a, T>),
}
#[derive(Debug, PartialEq, Clone)]
pub enum PlaceExpr<'a, T> {
    Var(&'a str),
    Property(Property<'a, T>),
    Index(Binary<'a, T>),
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
