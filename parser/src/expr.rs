use crate::expr::compound::Element;
use crate::expr::compound::Struct;
use crate::expr::control_flow::Block;
use crate::expr::control_flow::Break;
use crate::expr::control_flow::For;
use crate::expr::control_flow::Fun;
use crate::expr::control_flow::If;
use crate::expr::control_flow::While;
use crate::expr::operator::Assign;
use crate::expr::operator::Binary;
use crate::expr::operator::NamedArgCall;
use crate::expr::operator::Property;
use crate::expr::operator::Slice;
use crate::expr::operator::UnnamedArgCall;
use crate::expr::range::Range;

pub mod compound;
pub mod control_flow;
pub mod operator;
pub mod range;

#[derive(Debug, PartialEq, Clone)]
pub enum Expr<'a> {
    True,
    False,
    Null,

    Var(&'a str),
    UInt(u64),
    Float(f64),

    Break(Break<'a>),
    Continue(Option<&'a str>),
    Return(Option<Box<Expr<'a>>>),

    Plus(Box<Expr<'a>>),
    Minus(Box<Expr<'a>>),
    Ref(Box<Expr<'a>>),
    MutRef(Box<Expr<'a>>),
    Not(Box<Expr<'a>>),

    Add(Binary<'a>),
    Sub(Binary<'a>),
    Multiply(Binary<'a>),
    Div(Binary<'a>),
    FloorDiv(Binary<'a>),
    Mod(Binary<'a>),
    And(Binary<'a>),
    Or(Binary<'a>),
    LazyAnd(Binary<'a>),
    LazyOr(Binary<'a>),
    Equal(Binary<'a>),
    NotEqual(Binary<'a>),
    Greater(Binary<'a>),
    GreaterEqual(Binary<'a>),
    Less(Binary<'a>),
    LessEqual(Binary<'a>),
    Concatenate(Binary<'a>),
    NullOr(Binary<'a>),

    Assign(Assign<'a>),
    ParallelAssign(Box<[Assign<'a>]>),

    Array(Box<[Element<'a>]>),
    ArrayRange(Range<'a>),
    Struct(Struct<'a>),

    Property(Property<'a>),
    OptionalProperty(Property<'a>),
    Index(Binary<'a>),
    OptionalIndex(Binary<'a>),
    Slice(Slice<'a>),
    OptionalSlice(Slice<'a>),
    NamedArgCall(NamedArgCall<'a>),
    UnnamedArgCall(UnnamedArgCall<'a>),
    Deref(Box<Expr<'a>>),

    Block(Block<'a>),
    Fun(Fun<'a>),
    If(If<'a>),
    For(For<'a>),
    While(While<'a>),
    Loop(Block<'a>),
}
#[derive(Debug, PartialEq, Clone)]
pub enum PlaceExpr<'a> {
    Var(&'a str),
    Ref(Box<Expr<'a>>),
    Property(Property<'a>),
    Index(Binary<'a>),
}
impl<'a> PlaceExpr<'a> {
    pub fn from_expr(expr: Expr<'a>) -> Option<Self> {
        Some(match expr {
            Expr::Var(var) => Self::Var(var),
            Expr::Ref(expr) => Self::Ref(expr),
            Expr::Property(prop_expr) => Self::Property(prop_expr),
            Expr::Index(ind_expr) => Self::Index(ind_expr),
            _ => return None,
        })
    }
}
