use crate::{
    all_unique,
    pattern::Pattern,
    pretty_print::{
        bracket, line, multiline_sequence, postfix, prefix, sequence, PrettyPrint, PrettyPrintTree,
    },
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
    Record(Collection<Field<T>, T>),
    Tuple(Collection<Expr<T>, T>),

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
            ExprKind::ControlFlow(control_flow) => control_flow.to_pretty_print(),
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
#[derive(Debug, PartialEq, Clone)]
pub enum Collection<T, U> {
    Collection(Box<[T]>),
    WithSplat(WithSplat<T, U>),
}
impl<T> Collection<Field<T>, T> {
    pub fn all_name_unique(&self) -> bool {
        match self {
            Self::Collection(record) => all_unique(record.iter().map(|field| field.name.clone())),
            Self::WithSplat(record) => all_unique(
                record
                    .left
                    .iter()
                    .chain(record.right.iter())
                    .map(|field| field.name.clone()),
            ),
        }
    }
}
impl<T, U> PrettyPrint for Collection<T, U>
where
    T: PrettyPrint,
    U: PrettyPrintType,
{
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        match self {
            Self::Collection(tuple) => {
                let iter = tuple
                    .iter()
                    .map(T::to_pretty_print)
                    .map(|field| postfix(", ", field));
                bracket("(", ")", sequence(iter))
            }
            Self::WithSplat(tuple) => tuple.to_pretty_print(),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct WithSplat<T, U> {
    pub left: Box<[T]>,
    pub splat: Box<Expr<U>>,
    pub right: Box<[T]>,
}
impl<T, U> PrettyPrint for WithSplat<T, U>
where
    T: PrettyPrint,
    U: PrettyPrintType,
{
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        let iter = self
            .left
            .iter()
            .map(T::to_pretty_print)
            .chain(once(
                prefix("*", self.splat.to_pretty_print()) as Box<dyn PrettyPrintTree>
            ))
            .chain(self.right.iter().map(T::to_pretty_print))
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
impl<T: PrettyPrintType> PrettyPrint for ControlFlow<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        match self {
            ControlFlow::Block(block) => block.to_pretty_print(),
            ControlFlow::If(if_statement) => if_statement.to_pretty_print(),
            ControlFlow::For(for_statement) => for_statement.to_pretty_print(),
            ControlFlow::While(while_statement) => while_statement.to_pretty_print(),
            ControlFlow::Loop(loop_statement) => line([
                Box::new("loop ".to_string()),
                loop_statement.to_pretty_print(),
            ]),
            ControlFlow::Match(match_statement) => match_statement.to_pretty_print(),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Block<T> {
    pub statement: Box<[Statement<T>]>,
    pub expr: Option<Box<Expr<T>>>,
}
impl<T: PrettyPrintType> PrettyPrint for Block<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        if self.statement.is_empty() {
            match &self.expr {
                Some(expr) => bracket("{ ", " }", expr.to_pretty_print()),
                None => Box::new("{}".to_string()),
            }
        } else {
            let iter = self
                .statement
                .iter()
                .map(PrettyPrint::to_pretty_print)
                .map(|statement| postfix(";", statement))
                .chain(
                    self.expr
                        .iter()
                        .map(Box::as_ref)
                        .map(PrettyPrint::to_pretty_print),
                );
            bracket("{", "}", multiline_sequence(iter))
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct If<T> {
    pub condition: Box<Expr<T>>,
    pub body: Block<T>,
    pub else_part: Option<Box<ControlFlow<T>>>,
}
impl<T: PrettyPrintType> PrettyPrint for If<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        match &self.else_part {
            Some(else_part) => line([
                Box::new("if ".to_string()),
                self.condition.to_pretty_print(),
                Box::new(" ".to_string()),
                self.body.to_pretty_print(),
                Box::new(" else ".to_string()),
                else_part.to_pretty_print(),
            ]),
            None => line([
                Box::new("if ".to_string()),
                self.condition.to_pretty_print(),
                Box::new(" ".to_string()),
                self.body.to_pretty_print(),
            ]),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct For<T> {
    pub pattern: Pattern<T>,
    pub expr: Box<Expr<T>>,
    pub body: Block<T>,
}
impl<T: PrettyPrintType> PrettyPrint for For<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        line([
            Box::new("for ".to_string()),
            self.pattern.to_pretty_print(),
            Box::new(" in ".to_string()),
            self.expr.to_pretty_print(),
            Box::new(" ".to_string()),
            self.body.to_pretty_print(),
        ])
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct While<T> {
    pub condition: Box<Expr<T>>,
    pub body: Block<T>,
}
impl<T: PrettyPrintType> PrettyPrint for While<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        line([
            Box::new("while ".to_string()),
            self.condition.to_pretty_print(),
            Box::new(" ".to_string()),
            self.body.to_pretty_print(),
        ])
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Match<T> {
    pub expr: Box<Expr<T>>,
    pub arm: Box<[MatchArm<T>]>,
}
impl<T: PrettyPrintType> PrettyPrint for Match<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        let body = if self.arm.is_empty() {
            Box::new("{}".to_string())
        } else {
            let iter = self
                .arm
                .iter()
                .map(PrettyPrint::to_pretty_print)
                .map(|arm| postfix(",", arm));
            bracket("{ ", " }", multiline_sequence(iter))
        };
        line([
            Box::new("match ".to_string()),
            self.expr.to_pretty_print(),
            Box::new(" ".to_string()),
            body,
        ])
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct MatchArm<T> {
    pub pattern: Pattern<T>,
    pub expr: Expr<T>,
}
impl<T: PrettyPrintType> PrettyPrint for MatchArm<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        line([
            self.pattern.to_pretty_print(),
            Box::new(" => ".to_string()),
            self.expr.to_pretty_print(),
        ])
    }
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
    Record(Collection<Field<T>, T>),
    Tuple(Collection<Expr<T>, T>),
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
