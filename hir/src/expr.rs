use crate::{all_unique, pattern::Pattern, statement::Statement, Atom};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Literal {
    True,
    False,

    UInt(u64),
    Float(f64),
}
impl Literal {
    pub fn pretty_print(&self) -> String {
        match self {
            Self::True => "true".to_string(),
            Self::False => "false".to_string(),
            Self::UInt(num) => num.to_string(),
            Self::Float(num) => num.to_string(),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Expr<T> {
    pub expr: ExprKind<T>,
    pub ty: T,
}
impl<T> Expr<T> {
    pub fn field_name(&self) -> Option<Atom> {
        self.expr.field_name()
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum ExprKind<T> {
    Literal(Literal),

    Tag(Tag<T>),

    Assign(Box<[Assign<T>]>),

    Array(Box<[Element<T>]>),
    ArrayRange(Range<T>),

    Unit,
    Splat(Box<Expr<T>>),
    Record(Record<T>),
    Tuple(Tuple<T>),

    Unary(Unary<T>),
    Binary(Binary<T>),
    Place(PlaceExpr<T>),

    Call(Call<T>),

    ControlFlow(ControlFlow<T>),
    Fun(Fun<T>),
    Jump(Jump<T>),
}
impl<T> ExprKind<T> {
    pub fn field_name(&self) -> Option<Atom> {
        match self {
            Self::Tag(tag) => tag
                .expr
                .as_ref()
                .and_then(|expr| ExprKind::field_name(&expr.expr)),
            Self::Unary(unary) => unary.expr.field_name(),
            Self::Place(place) => place.field_name(),
            _ => None,
        }
    }
}
impl ExprKind<()> {
    pub fn into_untyped(self) -> Expr<()> {
        Expr { expr: self, ty: () }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum PlaceExpr<T> {
    Var(Atom),
    FieldAccess(FieldAccess<T>),
    Index(Index<T>),
    Slice(Slice<T>),
    Deref(Box<Expr<T>>),
    Len(Box<Expr<T>>),
}
impl<T> PlaceExpr<T> {
    pub fn field_name(&self) -> Option<Atom> {
        match self {
            Self::Var(var) => Some(var.clone()),
            Self::FieldAccess(field) => field.field_name(),
            Self::Deref(deref) => deref.field_name(),
            _ => None,
        }
    }
    pub fn var(&self) -> Option<Atom> {
        match self {
            PlaceExpr::Var(var) => Some(var.clone()),
            PlaceExpr::FieldAccess(FieldAccess { expr, name: _ })
            | PlaceExpr::Index(Index { expr, index: _ })
            | PlaceExpr::Slice(Slice { expr, range: _ })
            | PlaceExpr::Deref(expr)
            | PlaceExpr::Len(expr) => {
                let expr: &ExprKind<_> = &expr.expr;
                if let ExprKind::Place(place) = expr {
                    place.var()
                } else {
                    None
                }
            }
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Fun<T> {
    pub param: Pattern<T>,
    pub body: Box<Expr<T>>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Jump<T> {
    Break(Option<Box<Expr<T>>>),
    Continue,
    Return(Option<Box<Expr<T>>>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Unary<T> {
    pub kind: UnaryType,
    pub expr: Box<Expr<T>>,
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
pub struct Binary<T> {
    pub kind: BinaryType,
    pub left: Box<Expr<T>>,
    pub right: Box<Expr<T>>,
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
pub struct Index<T> {
    pub expr: Box<Expr<T>>,
    pub index: Box<Expr<T>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Element<T> {
    pub expr: Expr<T>,
    pub kind: ElementKind,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ElementKind {
    Element,
    Splat,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Tuple<T> {
    Tuple(Box<[Expr<T>]>),
    TupleWithSplat(TupleWithSplat<T>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct TupleWithSplat<T> {
    pub left: Box<[Expr<T>]>,
    pub splat: Box<Expr<T>>,
    pub right: Box<[Expr<T>]>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Record<T> {
    Record(Box<[Field<T>]>),
    RecordWithSplat(RecordWithSplat<T>),
}
impl<T> Record<T> {
    pub fn all_name_unique(&self) -> bool {
        match self {
            Self::Record(record) => all_unique(record.iter().map(|field| field.name.clone())),
            Self::RecordWithSplat(record) => all_unique(
                record
                    .left
                    .iter()
                    .chain(record.right.iter())
                    .map(|field| field.name.clone()),
            ),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct RecordWithSplat<T> {
    pub left: Box<[Field<T>]>,
    pub splat: Box<Expr<T>>,
    pub right: Box<[Field<T>]>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Field<T> {
    pub name: Atom,
    pub expr: Expr<T>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum ControlFlow<T> {
    Block(Block<T>),
    If(If<T>),
    For(For<T>),
    While(While<T>),
    Loop(Block<T>),
    Match(Match<T>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Block<T> {
    pub statement: Box<[Statement<T>]>,
    pub expr: Option<Box<Expr<T>>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct If<T> {
    pub condition: Box<Expr<T>>,
    pub body: Block<T>,
    pub else_part: Option<Box<ControlFlow<T>>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct For<T> {
    pub pattern: Pattern<T>,
    pub expr: Box<Expr<T>>,
    pub body: Block<T>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct While<T> {
    pub condition: Box<Expr<T>>,
    pub body: Block<T>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Match<T> {
    pub expr: Box<Expr<T>>,
    pub arm: Box<[MatchArm<T>]>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct MatchArm<T> {
    pub pattern: Pattern<T>,
    pub expr: Expr<T>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Assign<T> {
    pub place: PlaceExpr<T>,
    pub expr: Expr<T>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct FieldAccess<T> {
    pub expr: Box<Expr<T>>,
    pub name: Atom,
}
impl<T> FieldAccess<T> {
    pub fn field_name(&self) -> Option<Atom> {
        Some(self.name.clone())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Slice<T> {
    pub expr: Box<Expr<T>>,
    pub range: Range<T>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Call<T> {
    pub expr: Box<Expr<T>>,
    pub arg: Arg<T>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Arg<T> {
    Unit,
    Splat(Box<Expr<T>>),
    Record(Record<T>),
    Tuple(Tuple<T>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Tag<T> {
    pub tag: Atom,
    pub expr: Option<Box<Expr<T>>>,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BoundType {
    Inclusive,
    Exclusive,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Bound<T> {
    pub kind: BoundType,
    pub expr: Box<Expr<T>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Range<T> {
    pub left: Option<Bound<T>>,
    pub right: Option<Bound<T>>,
}
