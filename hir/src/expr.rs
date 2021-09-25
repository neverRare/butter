use crate::{
    pattern::{Pattern, Var},
    statement::Statement,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Literal {
    True,
    False,

    UInt(u64),
    Float(f64),
}
#[derive(Debug, PartialEq, Clone)]
pub enum Expr<'a, T> {
    Literal(Literal),

    Tag(Tag<'a, T>),

    Assign(Assign<'a, T>),
    ParallelAssign(Box<[Assign<'a, T>]>),

    Array(Box<[Element<'a, T>]>),
    ArrayRange(Range<'a, T>),

    Unit,
    Splat(Box<Expr<'a, T>>),
    Record(Record<'a, T>),
    Tuple(Tuple<'a, T>),

    Unary(Unary<'a, T>),
    Binary(Binary<'a, T>),
    Place(PlaceExpr<'a, T>),

    Call(Call<'a, T>),

    ControlFlow(ControlFlow<'a, T>),
    Fun(Fun<'a, T>),
    Jump(Jump<'a, T>),
}
impl<'a, T> Expr<'a, T> {
    pub fn field_name(&self) -> Option<&'a str> {
        match self {
            Self::Tag(tag) => todo!(),
            Self::Unary(unary) => todo!(),
            Self::Place(place) => place.field_name(),
            _ => None,
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum PlaceExpr<'a, T> {
    Var(&'a str),
    FieldAccess(FieldAccess<'a, T>),
    Index(Index<'a, T>),
    Slice(Slice<'a, T>),
    Deref(Box<Expr<'a, T>>),
    Len(Box<Expr<'a, T>>),
}
impl<'a, T> PlaceExpr<'a, T> {
    pub fn field_name(&self) -> Option<&'a str> {
        match self {
            Self::Var(var) => Some(var),
            Self::FieldAccess(field) => field.field_name(),
            Self::Deref(deref) => deref.field_name(),
            _ => None,
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Fun<'a, T> {
    pub param: Box<[Var<'a, T>]>,
    pub body: Box<Expr<'a, T>>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Jump<'a, T> {
    Break(Option<Box<Expr<'a, T>>>),
    Continue,
    Return(Option<Box<Expr<'a, T>>>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Unary<'a, T> {
    pub kind: UnaryType,
    pub expr: Box<Expr<'a, T>>,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnaryType {
    Minus,
    Ref,
    Not,
    Move,
    Clone,
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
#[derive(Debug, PartialEq, Clone)]
pub struct Element<'a, T> {
    pub expr: Expr<'a, T>,
    pub kind: ElementKind,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ElementKind {
    Element,
    Splat,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Tuple<'a, T> {
    Tuple(Box<[Expr<'a, T>]>),
    TupleWithSplat(TupleWithSplat<'a, T>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct TupleWithSplat<'a, T> {
    pub left: Box<[Expr<'a, T>]>,
    pub splat: Box<Expr<'a, T>>,
    pub right: Box<[Expr<'a, T>]>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Record<'a, T> {
    Record(Box<[Field<'a, T>]>),
    RecordWithSplat(RecordWithSplat<'a, T>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct RecordWithSplat<'a, T> {
    pub left: Box<[Field<'a, T>]>,
    pub splat: Box<Expr<'a, T>>,
    pub right: Box<[Field<'a, T>]>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Field<'a, T> {
    pub name: &'a str,
    pub expr: Expr<'a, T>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum ControlFlow<'a, T> {
    Block(Block<'a, T>),
    If(If<'a, T>),
    For(For<'a, T>),
    While(While<'a, T>),
    Loop(Block<'a, T>),
    Match(Match<'a, T>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Block<'a, T> {
    pub statement: Box<[Statement<'a, T>]>,
    pub expr: Option<Box<Expr<'a, T>>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct If<'a, T> {
    pub condition: Box<Expr<'a, T>>,
    pub body: Block<'a, T>,
    pub else_part: Option<Box<ControlFlow<'a, T>>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct For<'a, T> {
    pub pattern: Pattern<'a, T>,
    pub expr: Box<Expr<'a, T>>,
    pub body: Block<'a, T>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct While<'a, T> {
    pub condition: Box<Expr<'a, T>>,
    pub body: Block<'a, T>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Match<'a, T> {
    pub expr: Box<Expr<'a, T>>,
    pub arm: Box<[MatchArm<'a, T>]>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct MatchArm<'a, T> {
    pub pattern: Pattern<'a, T>,
    pub expr: Box<Expr<'a, T>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Assign<'a, T> {
    pub place: Box<PlaceExpr<'a, T>>,
    pub expr: Box<Expr<'a, T>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct FieldAccess<'a, T> {
    pub expr: Box<Expr<'a, T>>,
    pub name: &'a str,
}
impl<'a, T> FieldAccess<'a, T> {
    pub fn field_name(&self) -> Option<&'a str> {
        Some(self.name)
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Slice<'a, T> {
    pub expr: Box<Expr<'a, T>>,
    pub range: Range<'a, T>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Call<'a, T> {
    pub expr: Box<Expr<'a, T>>,
    pub arg: Arg<'a, T>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Arg<'a, T> {
    Unit,
    Splat(Box<Expr<'a, T>>),
    Record(Record<'a, T>),
    Tuple(Tuple<'a, T>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Tag<'a, T> {
    pub tag: &'a str,
    pub expr: Option<Box<Expr<'a, T>>>,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BoundType {
    Inclusive,
    Exclusive,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Bound<'a, T> {
    pub kind: BoundType,
    pub expr: Box<Expr<'a, T>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Range<'a, T> {
    pub left: Option<Bound<'a, T>>,
    pub right: Option<Bound<'a, T>>,
}
