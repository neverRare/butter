use crate::{
    all_unique,
    pattern::Pattern,
    pretty_print::{
        bracket, line, multiline_sequence, postfix, prefix, sequence, PrettyPrint, PrettyPrintTree,
    },
    statement::Statement,
    Atom, PrettyPrintType, TraverseType,
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
pub struct Expr<T: PrettyPrintType> {
    pub expr: ExprKind<T>,
    pub ty: T,
}
impl<T: PrettyPrintType> Expr<T> {
    pub fn field_name(&self) -> Option<Atom> {
        self.expr.field_name()
    }
    fn precedence(&self) -> u8
    where
        T: PrettyPrintType,
    {
        if T::TYPED {
            10
        } else {
            self.expr.precedence()
        }
    }
    fn to_auto_wrap(&self, precedence: u8) -> Box<dyn PrettyPrintTree>
    where
        T: PrettyPrintType,
    {
        let mut expr = self.to_pretty_print();
        if self.precedence() > precedence {
            expr = bracket("(", ")", expr);
        }
        expr
    }
}
impl<T: PrettyPrintType> TraverseType for Expr<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        mut for_type: impl FnMut(&mut Self::Type, &U) -> Result<(), E>,
        _for_scheme: impl FnMut(
            &mut <Self::Type as PrettyPrintType>::FunScheme,
            &mut U,
        ) -> Result<(), E>,
    ) -> Result<(), E> {
        for_type(&mut self.ty, data)?;
        todo!()
    }
}
impl<T: PrettyPrintType> PrettyPrint for Expr<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        let expr = self.expr.to_pretty_print();
        match self.ty.to_pretty_print() {
            Some(ty) => line([expr, Box::new(" : ".to_string()), ty]),
            None => expr,
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum ExprKind<T: PrettyPrintType> {
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
impl<T: PrettyPrintType> ExprKind<T> {
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
            ExprKind::Tag(_) => 2,
            ExprKind::Assign(_) => 8,
            ExprKind::Array(_) => 0,
            ExprKind::ArrayRange(_) => 0,
            ExprKind::Unit => 0,
            ExprKind::Splat(_) => 0,
            ExprKind::Record(_) => 0,
            ExprKind::Tuple(_) => 0,
            ExprKind::Unary(_) => 2,
            ExprKind::Binary(binary) => binary.kind.precedence(),
            ExprKind::Place(place) => place.precedence(),
            ExprKind::Call(_) => 1,
            ExprKind::ControlFlow(_) => 0,
            ExprKind::Fun(_) => 9,
            ExprKind::Jump(jump) => jump.precedence(),
        }
    }
}
impl<T: PrettyPrintType> PrettyPrint for ExprKind<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        match self {
            ExprKind::Literal(literal) => Box::new(literal.to_string()),
            ExprKind::Tag(tag) => tag.to_pretty_print(),
            ExprKind::Assign(assign) if assign.len() == 1 => assign[0].to_pretty_print(),
            ExprKind::Assign(assign) => sequence(
                assign
                    .iter()
                    .map(|assign| &assign.place)
                    .map(PlaceExpr::to_pretty_print)
                    .map(|place| postfix(", ", place))
                    .chain(once(
                        Box::new(" <- ".to_string()) as Box<dyn PrettyPrintTree>
                    ))
                    .chain(
                        assign
                            .iter()
                            .map(|assign| &assign.expr)
                            .map(Expr::to_pretty_print)
                            .map(|expr| postfix(", ", expr)),
                    ),
            ),
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
            ExprKind::Unary(unary) => unary.to_pretty_print(),
            ExprKind::Binary(binary) => binary.to_pretty_print(),
            ExprKind::Place(place) => place.to_pretty_print(),
            ExprKind::Call(call) => call.to_pretty_print(),
            ExprKind::ControlFlow(control_flow) => control_flow.to_pretty_print(),
            ExprKind::Fun(fun) => fun.to_pretty_print(),
            ExprKind::Jump(jump) => jump.to_pretty_print(),
        }
    }
}
impl ExprKind<()> {
    pub fn into_untyped(self) -> Expr<()> {
        Expr { expr: self, ty: () }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum PlaceExpr<T: PrettyPrintType> {
    Var(Atom),
    FieldAccess(FieldAccess<T>),
    Index(Index<T>),
    Slice(Slice<T>),
    Deref(Box<Expr<T>>),
    Len(Box<Expr<T>>),
}
impl<T: PrettyPrintType> PrettyPrint for PlaceExpr<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        match self {
            PlaceExpr::Var(var) => Box::new(var.to_string()),
            PlaceExpr::FieldAccess(field_access) => field_access.to_pretty_print(),
            PlaceExpr::Index(index) => index.to_pretty_print(),
            PlaceExpr::Slice(slice) => slice.to_pretty_print(),
            PlaceExpr::Deref(expr) => postfix("^", expr.to_auto_wrap(1)),
            PlaceExpr::Len(expr) => postfix(".len", expr.to_auto_wrap(1)),
        }
    }
}
impl<T: PrettyPrintType> PlaceExpr<T> {
    fn precedence(&self) -> u8 {
        match self {
            PlaceExpr::Var(_) => 0,
            _ => 1,
        }
    }
}
impl<T: PrettyPrintType> PlaceExpr<T> {
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
pub struct Fun<T: PrettyPrintType> {
    pub param: Pattern<T>,
    pub body: Box<Expr<T>>,
}
impl<T: PrettyPrintType> TraverseType for Fun<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        mut for_type: impl FnMut(&mut Self::Type, &U) -> Result<(), E>,
        mut for_scheme: impl FnMut(
            &mut <Self::Type as PrettyPrintType>::FunScheme,
            &mut U,
        ) -> Result<(), E>,
    ) -> Result<(), E> {
        self.param.traverse_type(data, &mut for_type, &mut for_scheme)?;
        self.body.traverse_type(data, for_type, for_scheme)?;
        Ok(())
    }
}
impl<T: PrettyPrintType> PrettyPrint for Fun<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        line([
            self.param.to_pretty_print(),
            Box::new(" => ".to_string()),
            self.body.to_auto_wrap(9),
        ])
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum Jump<T: PrettyPrintType> {
    Break(Option<Box<Expr<T>>>),
    Continue,
    Return(Option<Box<Expr<T>>>),
}
impl<T: PrettyPrintType> PrettyPrint for Jump<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        match self {
            Jump::Break(Some(expr)) => prefix("break ", expr.to_auto_wrap(9)),
            Jump::Break(None) => Box::new("break".to_string()),
            Jump::Continue => Box::new("continue".to_string()),
            Jump::Return(Some(expr)) => prefix("return ", expr.to_auto_wrap(9)),
            Jump::Return(None) => Box::new("return".to_string()),
        }
    }
}
impl<T: PrettyPrintType> Jump<T> {
    fn precedence(&self) -> u8 {
        match self {
            Jump::Break(_) => 9,
            Jump::Continue => 0,
            Jump::Return(_) => 9,
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Unary<T: PrettyPrintType> {
    pub kind: UnaryType,
    pub expr: Box<Expr<T>>,
}
impl<T: PrettyPrintType> PrettyPrint for Unary<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        let expr = self.expr.to_auto_wrap(2);
        let extra_space = match &self.kind {
            UnaryType::Clone => " ",
            _ => "",
        };
        line([Box::new(format!("{}{extra_space}", &self.kind)), expr])
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnaryType {
    Minus,
    Ref,
    Not,
    Move,
    Clone,
}
impl Display for UnaryType {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            UnaryType::Minus => "-",
            UnaryType::Ref => "&",
            UnaryType::Not => "!",
            UnaryType::Move => ">",
            UnaryType::Clone => "clone",
        };
        s.fmt(fmt)
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Binary<T: PrettyPrintType> {
    pub kind: BinaryType,
    pub left: Box<Expr<T>>,
    pub right: Box<Expr<T>>,
}
impl<T: PrettyPrintType> PrettyPrint for Binary<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        let precedence = self.kind.precedence();
        let left = self.left.to_auto_wrap(precedence);
        let right = self.right.to_auto_wrap(precedence);
        line([left, Box::new(format!(" {} ", &self.kind)), right])
    }
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
            Self::Multiply | Self::Div | Self::FloorDiv | Self::Mod => 3,
            Self::Add | Self::Sub | Self::Concatenate => 4,
            Self::Equal
            | Self::NotEqual
            | Self::Greater
            | Self::GreaterEqual
            | Self::Less
            | Self::LessEqual => 5,
            Self::And | Self::LazyAnd => 6,
            Self::Or | Self::LazyOr => 7,
        }
    }
}
impl Display for BinaryType {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            BinaryType::Add => "+",
            BinaryType::Sub => "-",
            BinaryType::Multiply => "*",
            BinaryType::Div => "/",
            BinaryType::FloorDiv => "//",
            BinaryType::Mod => "%",
            BinaryType::And => "&",
            BinaryType::Or => "|",
            BinaryType::LazyAnd => "&&",
            BinaryType::LazyOr => "||",
            BinaryType::Equal => "==",
            BinaryType::NotEqual => "!=",
            BinaryType::Greater => ">",
            BinaryType::GreaterEqual => ">=",
            BinaryType::Less => "<",
            BinaryType::LessEqual => "<=",
            BinaryType::Concatenate => "++",
        };
        s.fmt(fmt)
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Index<T: PrettyPrintType> {
    pub expr: Box<Expr<T>>,
    pub index: Box<Expr<T>>,
}
impl<T: PrettyPrintType> PrettyPrint for Index<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        line([
            self.expr.to_auto_wrap(1),
            bracket("[", "]", self.index.to_pretty_print()),
        ])
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Element<T: PrettyPrintType> {
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
pub enum Collection<T, U: PrettyPrintType> {
    Collection(Box<[T]>),
    WithSplat(WithSplat<T, U>),
}
impl<T: PrettyPrintType> Collection<Field<T>, T> {
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
pub struct WithSplat<T, U: PrettyPrintType> {
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
pub struct Field<T: PrettyPrintType> {
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
pub enum ControlFlow<T: PrettyPrintType> {
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
pub struct Block<T: PrettyPrintType> {
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
pub struct If<T: PrettyPrintType> {
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
pub struct For<T: PrettyPrintType> {
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
pub struct While<T: PrettyPrintType> {
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
pub struct Match<T: PrettyPrintType> {
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
pub struct MatchArm<T: PrettyPrintType> {
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
pub struct Assign<T: PrettyPrintType> {
    pub place: PlaceExpr<T>,
    pub expr: Expr<T>,
}
impl<T: PrettyPrintType> PrettyPrint for Assign<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        line([
            self.place.to_pretty_print(),
            Box::new(" <- ".to_string()),
            self.expr.to_auto_wrap(8),
        ])
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct FieldAccess<T: PrettyPrintType> {
    pub expr: Box<Expr<T>>,
    pub name: Atom,
}
impl<T: PrettyPrintType> FieldAccess<T> {
    pub fn field_name(&self) -> Option<Atom> {
        Some(self.name.clone())
    }
}
impl<T: PrettyPrintType> PrettyPrint for FieldAccess<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        line([
            self.expr.to_auto_wrap(1),
            Box::new(format!(".{}", &self.name)),
        ])
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Slice<T: PrettyPrintType> {
    pub expr: Box<Expr<T>>,
    pub range: Range<T>,
}
impl<T: PrettyPrintType> PrettyPrint for Slice<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        line([self.expr.to_auto_wrap(1), self.range.to_pretty_print()])
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Call<T: PrettyPrintType> {
    pub expr: Box<Expr<T>>,
    pub arg: Arg<T>,
}
impl<T: PrettyPrintType> PrettyPrint for Call<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        line([self.expr.to_auto_wrap(1), self.arg.to_pretty_print()])
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum Arg<T: PrettyPrintType> {
    Unit,
    Splat(Box<Expr<T>>),
    Record(Collection<Field<T>, T>),
    Tuple(Collection<Expr<T>, T>),
}
impl<T: PrettyPrintType> PrettyPrint for Arg<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        match self {
            Arg::Unit => Box::new("()".to_string()),
            Arg::Splat(expr) => bracket("(", ")", prefix("*", expr.to_pretty_print())),
            Arg::Record(record) => record.to_pretty_print(),
            Arg::Tuple(tuple) => tuple.to_pretty_print(),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Tag<T: PrettyPrintType> {
    pub tag: Atom,
    pub expr: Option<Box<Expr<T>>>,
}
impl<T: PrettyPrintType> PrettyPrint for Tag<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        match &self.expr {
            Some(expr) => {
                let pretty_print_tree = expr.to_auto_wrap(2);
                line([Box::new(format!("@{} ", &self.tag)), pretty_print_tree])
            }
            None => Box::new(format!("@{}", &self.tag)),
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BoundType {
    Inclusive,
    Exclusive,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Bound<T: PrettyPrintType> {
    pub kind: BoundType,
    pub expr: Box<Expr<T>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Range<T: PrettyPrintType> {
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
