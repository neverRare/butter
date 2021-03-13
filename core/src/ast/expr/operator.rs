use crate::ast::expr::compound::Arg;
use crate::ast::expr::range::Range;
use crate::ast::expr::Expr;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unary {
    Plus,
    Minus,
    Ref,
    Not,
    Clone,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Binary {
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
    NullOr,
    Assign,
}
pub struct UnaryExpr<'a> {
    pub op: Unary,
    pub expr: Box<Expr<'a>>,
}
pub struct BinaryExpr<'a> {
    pub op: Unary,
    pub left: Box<Expr<'a>>,
    pub right: Box<Expr<'a>>,
}
pub struct Property<'a> {
    pub expr: Box<Expr<'a>>,
    pub name: &'a str,
}
pub struct Slice<'a> {
    pub expr: Box<Expr<'a>>,
    pub range: Range<'a>,
}
pub struct Call<'a> {
    pub expr: Box<Expr<'a>>,
    pub args: Arg<'a>,
}
