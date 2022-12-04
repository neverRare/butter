use crate::{
    all_unique,
    pattern::Pattern,
    pretty_print::{bracket, line, postfix, prefix, sequence, PrettyPrint, PrettyPrintTree},
    statement::Statement,
    Atom, PrettyPrintType,
};
use std::{
    fmt::{self, Display, Formatter},
    iter::once,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Literal {
    True,
    False,

    UInt(u64),
    Float(f64),
}
impl Display for Literal {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::True => write!(fmt, "true")?,
            Self::False => write!(fmt, "false")?,
            Self::UInt(num) => write!(fmt, "{num}")?,
            Self::Float(num) => write!(fmt, "{num}")?,
        }
        Ok(())
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
    fn precedence(&self) -> u8
    where
        T: PrettyPrintType,
    {
        if T::TYPED {
            9
        } else {
            self.expr.precedence()
        }
    }
}
impl<T: PrettyPrintType> PrettyPrint for Expr<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        let pattern = self.expr.to_pretty_print();
        match self.ty.to_pretty_print() {
            Some(ty) => line([Box::new(ty), Box::new(" : ".to_string()), pattern]),
            None => pattern,
        }
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
    pub fn precedence(&self) -> u8 {
        match self {
            ExprKind::Literal(_) => 0,
            ExprKind::Tag(_) => 1,
            ExprKind::Assign(_) => 7,
            ExprKind::Array(_) => 0,
            ExprKind::ArrayRange(_) => 0,
            ExprKind::Unit => 0,
            ExprKind::Splat(_) => 0,
            ExprKind::Record(_) => 0,
            ExprKind::Tuple(_) => 0,
            ExprKind::Unary(_) => 1,
            ExprKind::Binary(binary) => binary.kind.precedence(),
            ExprKind::Place(place) => place.precedence(),
            ExprKind::Call(_) => 0,
            ExprKind::ControlFlow(_) => 0,
            ExprKind::Fun(_) => 8,
            ExprKind::Jump(jump) => jump.precedence(),
        }
    }
}
impl<T: PrettyPrintType> PrettyPrint for ExprKind<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        match self {
            ExprKind::Literal(literal) => Box::new(literal.to_string()),
            ExprKind::Tag(_) => todo!(),
            ExprKind::Assign(_) => todo!(),
            ExprKind::Array(array) => {
                let iter = array
                    .iter()
                    .map(Element::to_pretty_print)
                    .map(|element| postfix(", ", element));
                bracket("[", "]", sequence(iter))
            }
            ExprKind::ArrayRange(array) => array.to_pretty_print(),
            ExprKind::Unit => Box::new("()".to_string()),
            ExprKind::Splat(expr) => bracket("(", ")", prefix("*", expr.to_pretty_print())),
            ExprKind::Record(record) => record.to_pretty_print(),
            ExprKind::Tuple(tuple) => tuple.to_pretty_print(),
            ExprKind::Unary(_) => todo!(),
            ExprKind::Binary(_) => todo!(),
            ExprKind::Place(_) => todo!(),
            ExprKind::Call(_) => todo!(),
            ExprKind::ControlFlow(_) => todo!(),
            ExprKind::Fun(_) => todo!(),
            ExprKind::Jump(_) => todo!(),
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
    fn precedence(&self) -> u8 {
        match self {
            PlaceExpr::Var(_) => 0,
            _ => 1,
        }
    }
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
impl<T> Jump<T> {
    fn precedence(&self) -> u8 {
        match self {
            Jump::Break(_) => 8,
            Jump::Continue => 0,
            Jump::Return(_) => 8,
        }
    }
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
impl BinaryType {
    fn precedence(&self) -> u8 {
        match self {
            Self::Multiply | Self::Div | Self::FloorDiv | Self::Mod => 2,
            Self::Add | Self::Sub | Self::Concatenate => 3,
            Self::Equal
            | Self::NotEqual
            | Self::Greater
            | Self::GreaterEqual
            | Self::Less
            | Self::LessEqual => 4,
            Self::And | Self::LazyAnd => 5,
            Self::Or | Self::LazyOr => 6,
        }
    }
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
impl<T: PrettyPrintType> PrettyPrint for Element<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        let expr = self.expr.to_pretty_print();
        match self.kind {
            ElementKind::Element => expr,
            ElementKind::Splat => prefix("*", expr),
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ElementKind {
    Element,
    Splat,
}
// TODO: Tuple<T> and TupleWithSplat<T> is roughly the same with Record<T> and
// RecordWithSplat<T>, maybe generalized them
#[derive(Debug, PartialEq, Clone)]
pub enum Tuple<T> {
    Tuple(Box<[Expr<T>]>),
    TupleWithSplat(TupleWithSplat<T>),
}
impl<T: PrettyPrintType> PrettyPrint for Tuple<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        match self {
            Self::Tuple(tuple) => {
                let iter = tuple
                    .iter()
                    .map(Expr::to_pretty_print)
                    .map(|field| postfix(", ", field));
                bracket("(", ")", sequence(iter))
            }
            Self::TupleWithSplat(tuple) => tuple.to_pretty_print(),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct TupleWithSplat<T> {
    pub left: Box<[Expr<T>]>,
    pub splat: Box<Expr<T>>,
    pub right: Box<[Expr<T>]>,
}
impl<T: PrettyPrintType> PrettyPrint for TupleWithSplat<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        let iter = self
            .left
            .iter()
            .map(Expr::to_pretty_print)
            .chain(once(
                prefix("*", self.splat.to_pretty_print()) as Box<dyn PrettyPrintTree>
            ))
            .chain(self.right.iter().map(Expr::to_pretty_print))
            .map(|field| postfix(", ", field));
        bracket("(", ")", sequence(iter))
    }
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
impl<T: PrettyPrintType> PrettyPrint for Record<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        match self {
            Record::Record(record) => {
                let iter = record
                    .iter()
                    .map(Field::to_pretty_print)
                    .map(|field| postfix(", ", field));
                bracket("(", ")", sequence(iter))
            }
            Record::RecordWithSplat(record) => record.to_pretty_print(),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct RecordWithSplat<T> {
    pub left: Box<[Field<T>]>,
    pub splat: Box<Expr<T>>,
    pub right: Box<[Field<T>]>,
}
impl<T: PrettyPrintType> PrettyPrint for RecordWithSplat<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        let iter = self
            .left
            .iter()
            .map(Field::to_pretty_print)
            .chain(once(prefix("*", self.splat.to_pretty_print())))
            .chain(self.right.iter().map(Field::to_pretty_print))
            .map(|field| postfix(", ", field));
        bracket("(", ")", sequence(iter))
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Field<T> {
    pub name: Atom,
    pub expr: Expr<T>,
}
impl<T: PrettyPrintType> PrettyPrint for Field<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        line([
            Box::new(format!("{} = ", self.name)),
            self.expr.to_pretty_print(),
        ])
    }
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
impl<T: PrettyPrintType> PrettyPrint for Range<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        let range = match (&self.left, &self.right) {
            (None, None) => Box::new("..".to_string()) as Box<dyn PrettyPrintTree>,
            (None, Some(range)) => {
                let expr = range.expr.to_pretty_print();
                let op = match range.kind {
                    BoundType::Inclusive => "..",
                    BoundType::Exclusive => ".<",
                };
                prefix(op, expr)
            }
            (Some(range), None) => {
                let expr = range.expr.to_pretty_print();
                let op = match range.kind {
                    BoundType::Inclusive => "..",
                    BoundType::Exclusive => ">.",
                };
                postfix(op, expr)
            }
            (Some(left), Some(right)) => {
                let op = match (left.kind, right.kind) {
                    (BoundType::Inclusive, BoundType::Inclusive) => "..",
                    (BoundType::Inclusive, BoundType::Exclusive) => ".<",
                    (BoundType::Exclusive, BoundType::Inclusive) => ">.",
                    (BoundType::Exclusive, BoundType::Exclusive) => "><",
                };
                let left = left.expr.to_pretty_print();
                let right = right.expr.to_pretty_print();
                line([left, Box::new(op.to_string()), right])
            }
        };
        bracket("[", "]", range)
    }
}
