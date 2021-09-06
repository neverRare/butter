use crate::{pattern::Pattern, statement::Statement};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Literal {
    True,
    False,
    Void,

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
    Record(Record<'a, T>),

    Unary(Unary<'a, T>),
    Binary(Binary<'a, T>),
    Place(PlaceExpr<'a, T>),

    NamedArgCall(NamedArgCall<'a, T>),
    UnnamedArgCall(UnnamedArgCall<'a, T>),

    ControlFlow(ControlFlow<'a, T>),
    Fun(Fun<'a, T>),
    Jump(Jump<'a, T>),
}
#[derive(Debug, PartialEq, Clone)]
pub enum PlaceExpr<'a, T> {
    Var(&'a str),
    Property(Property<'a, T>),
    Index(Index<'a, T>),
    Slice(Slice<'a, T>),
    Deref(Box<Expr<'a, T>>),
    Len(Box<Expr<'a, T>>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Fun<'a, T> {
    // TODO: preserve order
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
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Record<'a, T> {
    // TODO: preserve order
    pub splats: Box<[Expr<'a, T>]>,
    pub fields: HashMap<&'a str, Expr<'a, T>>,
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
pub struct Property<'a, T> {
    pub expr: Box<Expr<'a, T>>,
    pub name: &'a str,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Slice<'a, T> {
    pub expr: Box<Expr<'a, T>>,
    pub range: Range<'a, T>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct NamedArgCall<'a, T> {
    pub expr: Box<Expr<'a, T>>,
    pub args: Record<'a, T>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct UnnamedArgCall<'a, T> {
    pub expr: Box<Expr<'a, T>>,
    pub args: Box<[Expr<'a, T>]>,
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
