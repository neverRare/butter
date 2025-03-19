use crate::{
    Atom, PrettyPrintType, TraverseType, all_unique, bracket, intersperse_with_line,
    intersperse_with_space, pattern::Pattern, statement::Statement,
};
use pretty::BoxDoc;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Literal {
    UInt(u64),
    Float(f64),
}
impl Display for Literal {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
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
        if T::TYPED { 10 } else { self.expr.precedence() }
    }
    fn to_auto_wrap(&self, precedence: u8) -> BoxDoc
    where
        T: PrettyPrintType,
    {
        let expr = self.to_doc();
        if self.precedence() > precedence {
            bracket("(", ")", expr)
        } else {
            expr
        }
    }
    pub fn to_doc(&self) -> BoxDoc {
        let expr = self.expr.to_doc();
        match self.ty.to_doc() {
            Some(ty) => intersperse_with_space([expr, BoxDoc::text(":"), ty]),
            None => expr,
        }
    }
}
impl<T: PrettyPrintType> TraverseType for Expr<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        for_type(&mut self.ty, data)?;
        match &mut self.expr {
            ExprKind::Literal(_) => (),
            ExprKind::Tag(tag) => tag.traverse_type(data, for_type, for_scheme)?,
            ExprKind::Assign(assign) => {
                for assign in assign.iter_mut() {
                    assign.traverse_type(data, for_type, for_scheme)?;
                }
            }
            ExprKind::Array(array) => {
                for elem in array.iter_mut() {
                    elem.traverse_type(data, for_type, for_scheme)?;
                }
            }
            ExprKind::ArrayRange(range) => range.traverse_type(data, for_type, for_scheme)?,
            ExprKind::Unit => (),
            ExprKind::Splat(expr) => expr.traverse_type(data, for_type, for_scheme)?,
            ExprKind::Record(record) => record.traverse_type(data, for_type, for_scheme)?,
            ExprKind::Tuple(tuple) => tuple.traverse_type(data, for_type, for_scheme)?,
            ExprKind::Unary(unary) => unary.traverse_type(data, for_type, for_scheme)?,
            ExprKind::Binary(binary) => binary.traverse_type(data, for_type, for_scheme)?,
            ExprKind::Place(place) => place.traverse_type(data, for_type, for_scheme)?,
            ExprKind::Call(call) => call.traverse_type(data, for_type, for_scheme)?,
            ExprKind::ControlFlow(control_flow) => {
                control_flow.traverse_type(data, for_type, for_scheme)?
            }
            ExprKind::Fun(fun) => fun.traverse_type(data, for_type, for_scheme)?,
            ExprKind::Jump(jump) => jump.traverse_type(data, for_type, for_scheme)?,
        }
        Ok(())
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
    pub fn to_doc(&self) -> BoxDoc {
        match self {
            ExprKind::Literal(literal) => BoxDoc::as_string(literal),
            ExprKind::Tag(tag) => tag.to_doc(),
            ExprKind::Assign(assign) if assign.len() == 1 => assign[0].to_doc(),
            ExprKind::Assign(assign) => intersperse_with_space(
                assign
                    .iter()
                    .map(|assign| &assign.place)
                    .map(PlaceExpr::to_doc)
                    .map(|place| BoxDoc::concat([place, BoxDoc::text(",")]))
                    .chain([BoxDoc::text("<-")])
                    .chain(
                        assign
                            .iter()
                            .map(|assign| &assign.expr)
                            .map(Expr::to_doc)
                            .map(|expr| BoxDoc::concat([expr, BoxDoc::text(",")])),
                    ),
            ),
            ExprKind::Array(array) => {
                let iter = array
                    .iter()
                    .map(Element::to_doc)
                    .map(|element| BoxDoc::concat([element, BoxDoc::text(",")]));
                bracket("[", "]", intersperse_with_line(iter))
            }
            ExprKind::ArrayRange(array) => array.to_doc(),
            ExprKind::Unit => BoxDoc::text("()"),
            ExprKind::Splat(expr) => {
                bracket("(", ")", BoxDoc::concat([BoxDoc::text("*"), expr.to_doc()]))
            }
            ExprKind::Record(record) => record.to_doc(Field::to_doc),
            ExprKind::Tuple(tuple) => tuple.to_doc(Expr::to_doc),
            ExprKind::Unary(unary) => unary.to_doc(),
            ExprKind::Binary(binary) => binary.to_doc(),
            ExprKind::Place(place) => place.to_doc(),
            ExprKind::Call(call) => call.to_doc(),
            ExprKind::ControlFlow(control_flow) => control_flow.to_doc(),
            ExprKind::Fun(fun) => fun.to_doc(),
            ExprKind::Jump(jump) => jump.to_doc(),
        }
    }
}
impl<T: PrettyPrintType> TraverseType for ExprKind<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        match self {
            ExprKind::Literal(_) => (),
            ExprKind::Tag(tag) => tag.traverse_type(data, for_type, for_scheme)?,
            ExprKind::Assign(assign) => {
                for assign in assign.iter_mut() {
                    assign.traverse_type(data, for_type, for_scheme)?;
                }
            }
            ExprKind::Array(array) => {
                for element in array.iter_mut() {
                    element.traverse_type(data, for_type, for_scheme)?;
                }
            }
            ExprKind::ArrayRange(range) => range.traverse_type(data, for_type, for_scheme)?,
            ExprKind::Unit => (),
            ExprKind::Splat(expr) => expr.traverse_type(data, for_type, for_scheme)?,
            ExprKind::Record(record) => record.traverse_type(data, for_type, for_scheme)?,
            ExprKind::Tuple(tuple) => tuple.traverse_type(data, for_type, for_scheme)?,
            ExprKind::Unary(unary) => unary.traverse_type(data, for_type, for_scheme)?,
            ExprKind::Binary(binary) => binary.traverse_type(data, for_type, for_scheme)?,
            ExprKind::Place(place) => place.traverse_type(data, for_type, for_scheme)?,
            ExprKind::Call(call) => call.traverse_type(data, for_type, for_scheme)?,
            ExprKind::ControlFlow(control_flow) => {
                control_flow.traverse_type(data, for_type, for_scheme)?
            }
            ExprKind::Fun(fun) => fun.traverse_type(data, for_type, for_scheme)?,
            ExprKind::Jump(jump) => jump.traverse_type(data, for_type, for_scheme)?,
        }
        Ok(())
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
impl<T: PrettyPrintType> PlaceExpr<T> {
    pub fn to_doc(&self) -> BoxDoc {
        match self {
            PlaceExpr::Var(var) => BoxDoc::text(var as &str),
            PlaceExpr::FieldAccess(field_access) => field_access.to_doc(),
            PlaceExpr::Index(index) => index.to_doc(),
            PlaceExpr::Slice(slice) => slice.to_doc(),
            PlaceExpr::Deref(expr) => BoxDoc::concat([expr.to_auto_wrap(1), BoxDoc::text("^")]),
            PlaceExpr::Len(expr) => BoxDoc::concat([expr.to_auto_wrap(1), BoxDoc::text(".len")]),
        }
    }
}
impl<T: PrettyPrintType> TraverseType for PlaceExpr<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        match self {
            PlaceExpr::Var(_) => (),
            PlaceExpr::FieldAccess(field_access) => {
                field_access.traverse_type(data, for_type, for_scheme)?
            }
            PlaceExpr::Index(index) => index.traverse_type(data, for_type, for_scheme)?,
            PlaceExpr::Slice(slice) => slice.traverse_type(data, for_type, for_scheme)?,
            PlaceExpr::Deref(expr) => expr.traverse_type(data, for_type, for_scheme)?,
            PlaceExpr::Len(expr) => expr.traverse_type(data, for_type, for_scheme)?,
        }
        Ok(())
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
impl<T: PrettyPrintType> Fun<T> {
    pub fn to_doc(&self) -> BoxDoc {
        intersperse_with_line([
            self.param.to_doc(),
            BoxDoc::text("=>"),
            self.body.to_auto_wrap(9),
        ])
    }
}
impl<T: PrettyPrintType> TraverseType for Fun<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.param.traverse_type(data, for_type, for_scheme)?;
        self.body.traverse_type(data, for_type, for_scheme)?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum Jump<T: PrettyPrintType> {
    Break(Option<Box<Expr<T>>>),
    Continue,
    Return(Option<Box<Expr<T>>>),
}
impl<T: PrettyPrintType> Jump<T> {
    pub fn to_doc(&self) -> BoxDoc {
        match self {
            Jump::Break(Some(expr)) => {
                intersperse_with_space([BoxDoc::text("break"), expr.to_auto_wrap(9)])
            }
            Jump::Break(None) => BoxDoc::text("break"),
            Jump::Continue => BoxDoc::text("continue"),
            Jump::Return(Some(expr)) => {
                intersperse_with_space([BoxDoc::text("return"), expr.to_auto_wrap(9)])
            }
            Jump::Return(None) => BoxDoc::text("return"),
        }
    }
}
impl<T: PrettyPrintType> TraverseType for Jump<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        match self {
            Jump::Break(expr) => expr
                .as_mut()
                .map(|expr| expr.traverse_type(data, for_type, for_scheme))
                .unwrap_or(Ok(()))?,
            Jump::Continue => (),
            Jump::Return(expr) => expr
                .as_mut()
                .map(|expr| expr.traverse_type(data, for_type, for_scheme))
                .unwrap_or(Ok(()))?,
        }
        Ok(())
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
impl<T: PrettyPrintType> Unary<T> {
    pub fn to_doc(&self) -> BoxDoc {
        let extra_space = match &self.kind {
            UnaryType::Not => BoxDoc::space(),
            _ => BoxDoc::nil(),
        };
        BoxDoc::concat([
            BoxDoc::text(self.kind.as_static_str()),
            extra_space,
            self.expr.to_auto_wrap(2),
        ])
    }
}
impl<T: PrettyPrintType> TraverseType for Unary<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.expr.traverse_type(data, for_type, for_scheme)
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnaryType {
    Minus,
    Ref,
    Not,
    Move,
}
impl UnaryType {
    fn as_static_str(self) -> &'static str {
        match self {
            UnaryType::Minus => "-",
            UnaryType::Ref => "&",
            UnaryType::Not => "not",
            UnaryType::Move => ">",
        }
    }
}
impl Display for UnaryType {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        self.as_static_str().fmt(fmt)
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Binary<T: PrettyPrintType> {
    pub kind: BinaryType,
    pub left: Box<Expr<T>>,
    pub right: Box<Expr<T>>,
}
impl<T: PrettyPrintType> Binary<T> {
    pub fn to_doc(&self) -> BoxDoc {
        let precedence = self.kind.precedence();
        let left = self.left.to_auto_wrap(precedence);
        let right = self.right.to_auto_wrap(precedence);
        intersperse_with_space([left, BoxDoc::text(self.kind.as_static_str()), right])
    }
}
impl<T: PrettyPrintType> TraverseType for Binary<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.left.traverse_type(data, for_type, for_scheme)?;
        self.right.traverse_type(data, for_type, for_scheme)?;
        Ok(())
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
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Concatenate,
}
impl BinaryType {
    fn precedence(self) -> u8 {
        match self {
            Self::Multiply | Self::Div | Self::FloorDiv | Self::Mod => 3,
            Self::Add | Self::Sub | Self::Concatenate => 4,
            Self::Equal
            | Self::NotEqual
            | Self::Greater
            | Self::GreaterEqual
            | Self::Less
            | Self::LessEqual => 5,
            Self::And => 6,
            Self::Or => 7,
        }
    }
    fn as_static_str(self) -> &'static str {
        match self {
            BinaryType::Add => "+",
            BinaryType::Sub => "-",
            BinaryType::Multiply => "*",
            BinaryType::Div => "/",
            BinaryType::FloorDiv => "//",
            BinaryType::Mod => "%",
            BinaryType::And => "and",
            BinaryType::Or => "or",
            BinaryType::Equal => "==",
            BinaryType::NotEqual => "/=",
            BinaryType::Greater => ">",
            BinaryType::GreaterEqual => ">=",
            BinaryType::Less => "<",
            BinaryType::LessEqual => "<=",
            BinaryType::Concatenate => "++",
        }
    }
}
impl Display for BinaryType {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        self.as_static_str().fmt(fmt)
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Index<T: PrettyPrintType> {
    pub expr: Box<Expr<T>>,
    pub index: Box<Expr<T>>,
}
impl<T: PrettyPrintType> Index<T> {
    pub fn to_doc(&self) -> BoxDoc {
        BoxDoc::concat([
            self.expr.to_auto_wrap(1),
            bracket("[", "]", self.index.to_doc()),
        ])
    }
}
impl<T: PrettyPrintType> TraverseType for Index<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.expr.traverse_type(data, for_type, for_scheme)?;
        self.index.traverse_type(data, for_type, for_scheme)?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Element<T: PrettyPrintType> {
    pub expr: Expr<T>,
    pub kind: ElementKind,
}
impl<T: PrettyPrintType> Element<T> {
    pub fn to_doc(&self) -> BoxDoc {
        let expr = self.expr.to_doc();
        match self.kind {
            ElementKind::Element => expr,
            ElementKind::Splat => BoxDoc::concat([BoxDoc::text("*"), expr]),
        }
    }
}
impl<T: PrettyPrintType> TraverseType for Element<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.expr.traverse_type(data, for_type, for_scheme)
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
impl<T, U> Collection<T, U>
where
    U: PrettyPrintType,
{
    pub fn to_doc<'a>(&'a self, mapper: impl for<'b> Fn(&'b T) -> BoxDoc<'b>) -> BoxDoc<'a> {
        match self {
            Self::Collection(tuple) => {
                let iter = tuple
                    .iter()
                    .map(mapper)
                    .map(|field| BoxDoc::concat([field, BoxDoc::text(",")]));
                bracket("(", ")", intersperse_with_line(iter))
            }
            Self::WithSplat(tuple) => tuple.to_doc(mapper),
        }
    }
}
impl<T: TraverseType> TraverseType for Collection<T, T::Type> {
    type Type = T::Type;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        match self {
            Collection::Collection(collection) => {
                for element in collection.iter_mut() {
                    element.traverse_type(data, for_type, for_scheme)?;
                }
            }
            Collection::WithSplat(_) => todo!(),
        }
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct WithSplat<T, U: PrettyPrintType> {
    pub left: Box<[T]>,
    pub splat: Box<Expr<U>>,
    pub right: Box<[T]>,
}
impl<T, U> WithSplat<T, U>
where
    U: PrettyPrintType,
{
    pub fn to_doc<'a>(&'a self, mapper: impl for<'b> Fn(&'b T) -> BoxDoc<'b>) -> BoxDoc<'a> {
        let iter = self
            .left
            .iter()
            .map(&mapper)
            .chain([BoxDoc::concat([BoxDoc::text("*"), self.splat.to_doc()])])
            .chain(self.right.iter().map(&mapper))
            .map(|field| BoxDoc::concat([field, BoxDoc::text(",")]));
        bracket("(", ")", intersperse_with_line(iter))
    }
}
impl<T: TraverseType> TraverseType for WithSplat<T, T::Type> {
    type Type = T::Type;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        for element in self.left.iter_mut() {
            element.traverse_type(data, for_type, for_scheme)?;
        }
        self.splat.traverse_type(data, for_type, for_scheme)?;
        for element in self.right.iter_mut() {
            element.traverse_type(data, for_type, for_scheme)?;
        }
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Field<T: PrettyPrintType> {
    pub name: Atom,
    pub expr: Expr<T>,
}
impl<T: PrettyPrintType> Field<T> {
    pub fn to_doc(&self) -> BoxDoc {
        intersperse_with_space([
            BoxDoc::text(&self.name as &str),
            BoxDoc::text("="),
            self.expr.to_doc(),
        ])
    }
}
impl<T: PrettyPrintType> TraverseType for Field<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.expr.traverse_type(data, for_type, for_scheme)
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
impl<T: PrettyPrintType> ControlFlow<T> {
    pub fn to_doc(&self) -> BoxDoc {
        match self {
            ControlFlow::Block(block) => block.to_doc(),
            ControlFlow::If(if_statement) => if_statement.to_doc(),
            ControlFlow::For(for_statement) => for_statement.to_doc(),
            ControlFlow::While(while_statement) => while_statement.to_doc(),
            ControlFlow::Loop(loop_statement) => {
                intersperse_with_space([BoxDoc::text("loop"), loop_statement.to_doc()])
            }
            ControlFlow::Match(match_statement) => match_statement.to_doc(),
        }
    }
}
impl<T: PrettyPrintType> TraverseType for ControlFlow<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        match self {
            ControlFlow::Block(block) => block.traverse_type(data, for_type, for_scheme)?,
            ControlFlow::If(if_statement) => {
                if_statement.traverse_type(data, for_type, for_scheme)?
            }
            ControlFlow::For(for_statement) => {
                for_statement.traverse_type(data, for_type, for_scheme)?
            }
            ControlFlow::While(while_statement) => {
                while_statement.traverse_type(data, for_type, for_scheme)?
            }
            ControlFlow::Loop(block) => block.traverse_type(data, for_type, for_scheme)?,
            ControlFlow::Match(match_statement) => {
                match_statement.traverse_type(data, for_type, for_scheme)?
            }
        }
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Block<T: PrettyPrintType> {
    pub statement: Box<[Statement<T>]>,
    pub expr: Option<Box<Expr<T>>>,
}
impl<T: PrettyPrintType> Block<T> {
    pub fn to_doc(&self) -> BoxDoc {
        if self.statement.is_empty() {
            match &self.expr {
                Some(expr) => bracket("{", "}", expr.to_doc()),
                None => BoxDoc::text("{}"),
            }
        } else {
            let iter = self
                .statement
                .iter()
                .map(Statement::to_doc)
                .map(|statement| BoxDoc::concat([statement, BoxDoc::text(";")]))
                .chain(self.expr.iter().map(Box::as_ref).map(Expr::to_doc));
            bracket("{", "}", intersperse_with_line(iter))
        }
    }
}
impl<T: PrettyPrintType> TraverseType for Block<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        for statement in self.statement.iter_mut() {
            statement.traverse_type(data, for_type, for_scheme)?;
        }
        self.expr
            .as_mut()
            .map(|expr| expr.traverse_type(data, for_type, for_scheme))
            .unwrap_or(Ok(()))?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct If<T: PrettyPrintType> {
    pub condition: Box<Expr<T>>,
    pub body: Block<T>,
    pub else_part: Option<Box<ControlFlow<T>>>,
}
impl<T: PrettyPrintType> If<T> {
    pub fn to_doc(&self) -> BoxDoc {
        match &self.else_part {
            Some(else_part) => intersperse_with_space([
                BoxDoc::text("if"),
                self.condition.to_doc(),
                self.body.to_doc(),
                BoxDoc::text("else"),
                else_part.to_doc(),
            ]),
            None => intersperse_with_space([
                BoxDoc::text("if"),
                self.condition.to_doc(),
                self.body.to_doc(),
            ]),
        }
    }
}
impl<T: PrettyPrintType> TraverseType for If<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.condition.traverse_type(data, for_type, for_scheme)?;
        self.body.traverse_type(data, for_type, for_scheme)?;
        self.else_part
            .as_mut()
            .map(|else_part| else_part.traverse_type(data, for_type, for_scheme))
            .unwrap_or(Ok(()))?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct For<T: PrettyPrintType> {
    pub pattern: Pattern<T>,
    pub expr: Box<Expr<T>>,
    pub body: Block<T>,
}
impl<T: PrettyPrintType> For<T> {
    pub fn to_doc(&self) -> BoxDoc {
        intersperse_with_space([
            BoxDoc::text("for"),
            self.pattern.to_doc(),
            BoxDoc::text("in"),
            self.expr.to_doc(),
            self.body.to_doc(),
        ])
    }
}
impl<T: PrettyPrintType> TraverseType for For<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.pattern.traverse_type(data, for_type, for_scheme)?;
        self.expr.traverse_type(data, for_type, for_scheme)?;
        self.body.traverse_type(data, for_type, for_scheme)?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct While<T: PrettyPrintType> {
    pub condition: Box<Expr<T>>,
    pub body: Block<T>,
}
impl<T: PrettyPrintType> While<T> {
    pub fn to_doc(&self) -> BoxDoc {
        intersperse_with_space([
            BoxDoc::text("while"),
            self.condition.to_doc(),
            self.body.to_doc(),
        ])
    }
}
impl<T: PrettyPrintType> TraverseType for While<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.condition.traverse_type(data, for_type, for_scheme)?;
        self.body.traverse_type(data, for_type, for_scheme)?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Match<T: PrettyPrintType> {
    pub expr: Box<Expr<T>>,
    pub arm: Box<[MatchArm<T>]>,
}
impl<T: PrettyPrintType> Match<T> {
    pub fn to_doc(&self) -> BoxDoc {
        let body = if self.arm.is_empty() {
            BoxDoc::text("{}")
        } else {
            let iter = self
                .arm
                .iter()
                .map(MatchArm::to_doc)
                .map(|arm| BoxDoc::concat([arm, BoxDoc::text(",")]));
            bracket("{ ", " }", intersperse_with_line(iter))
        };
        intersperse_with_space([BoxDoc::text("match"), self.expr.to_doc(), body])
    }
}
impl<T: PrettyPrintType> TraverseType for Match<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.expr.traverse_type(data, for_type, for_scheme)?;
        for arm in self.arm.iter_mut() {
            arm.traverse_type(data, for_type, for_scheme)?;
        }
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct MatchArm<T: PrettyPrintType> {
    pub pattern: Pattern<T>,
    pub expr: Expr<T>,
}
impl<T: PrettyPrintType> MatchArm<T> {
    pub fn to_doc(&self) -> BoxDoc {
        intersperse_with_space([
            self.pattern.to_doc(),
            BoxDoc::text("=>"),
            self.expr.to_doc(),
        ])
    }
}
impl<T: PrettyPrintType> TraverseType for MatchArm<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.pattern.traverse_type(data, for_type, for_scheme)?;
        self.expr.traverse_type(data, for_type, for_scheme)?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Assign<T: PrettyPrintType> {
    pub place: PlaceExpr<T>,
    pub expr: Expr<T>,
}
impl<T: PrettyPrintType> Assign<T> {
    pub fn to_doc(&self) -> BoxDoc {
        intersperse_with_space([
            self.place.to_doc(),
            BoxDoc::text("<-"),
            self.expr.to_auto_wrap(8),
        ])
    }
}
impl<T: PrettyPrintType> TraverseType for Assign<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.place.traverse_type(data, for_type, for_scheme)?;
        self.expr.traverse_type(data, for_type, for_scheme)?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct FieldAccess<T: PrettyPrintType> {
    pub expr: Box<Expr<T>>,
    pub name: Atom,
}
impl<T: PrettyPrintType> TraverseType for FieldAccess<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.expr.traverse_type(data, for_type, for_scheme)
    }
}
impl<T: PrettyPrintType> FieldAccess<T> {
    pub fn field_name(&self) -> Option<Atom> {
        Some(self.name.clone())
    }
    pub fn to_doc(&self) -> BoxDoc {
        BoxDoc::concat([
            self.expr.to_auto_wrap(1),
            BoxDoc::text("."),
            BoxDoc::text(&self.name as &str),
        ])
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Slice<T: PrettyPrintType> {
    pub expr: Box<Expr<T>>,
    pub range: Range<T>,
}
impl<T: PrettyPrintType> Slice<T> {
    pub fn to_doc(&self) -> BoxDoc {
        BoxDoc::concat([self.expr.to_auto_wrap(1), self.range.to_doc()])
    }
}
impl<T: PrettyPrintType> TraverseType for Slice<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.expr.traverse_type(data, for_type, for_scheme)?;
        self.range.traverse_type(data, for_type, for_scheme)?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Call<T: PrettyPrintType> {
    pub expr: Box<Expr<T>>,
    pub arg: Arg<T>,
}
impl<T: PrettyPrintType> Call<T> {
    pub fn to_doc(&self) -> BoxDoc {
        BoxDoc::concat([self.expr.to_auto_wrap(1), self.arg.to_doc()])
    }
}
impl<T: PrettyPrintType> TraverseType for Call<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.expr.traverse_type(data, for_type, for_scheme)?;
        self.arg.traverse_type(data, for_type, for_scheme)?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum Arg<T: PrettyPrintType> {
    Unit,
    Splat(Box<Expr<T>>),
    Record(Collection<Field<T>, T>),
    Tuple(Collection<Expr<T>, T>),
}
impl<T: PrettyPrintType> Arg<T> {
    pub fn to_doc(&self) -> BoxDoc {
        match self {
            Arg::Unit => BoxDoc::text("()"),
            Arg::Splat(expr) => {
                bracket("(", ")", BoxDoc::concat([BoxDoc::text("*"), expr.to_doc()]))
            }
            Arg::Record(record) => record.to_doc(Field::to_doc),
            Arg::Tuple(tuple) => tuple.to_doc(Expr::to_doc),
        }
    }
}
impl<T: PrettyPrintType> TraverseType for Arg<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        match self {
            Arg::Unit => (),
            Arg::Splat(expr) => expr.traverse_type(data, for_type, for_scheme)?,
            Arg::Record(record) => record.traverse_type(data, for_type, for_scheme)?,
            Arg::Tuple(tuple) => tuple.traverse_type(data, for_type, for_scheme)?,
        }
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Tag<T: PrettyPrintType> {
    pub tag: Atom,
    pub expr: Option<Box<Expr<T>>>,
}
impl<T: PrettyPrintType> Tag<T> {
    pub fn to_doc(&self) -> BoxDoc {
        let expr = match &self.expr {
            Some(expr) => {
                let expr = expr.to_doc();
                if T::TYPED {
                    expr
                } else {
                    bracket("(", ")", expr)
                }
            }
            None => BoxDoc::nil(),
        };
        intersperse_with_space([
            BoxDoc::concat([BoxDoc::text("@"), BoxDoc::text(&self.tag as &str)]),
            expr,
        ])
    }
}
impl<T: PrettyPrintType> TraverseType for Tag<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.expr
            .as_mut()
            .map(|expr| expr.traverse_type(data, for_type, for_scheme))
            .unwrap_or(Ok(()))
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
impl<T: PrettyPrintType> TraverseType for Bound<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.expr.traverse_type(data, for_type, for_scheme)
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Range<T: PrettyPrintType> {
    pub left: Option<Bound<T>>,
    pub right: Option<Bound<T>>,
}
impl<T: PrettyPrintType> Range<T> {
    pub fn to_doc(&self) -> BoxDoc {
        let range = match (&self.left, &self.right) {
            (None, None) => BoxDoc::text(".."),
            (None, Some(range)) => {
                let expr = range.expr.to_doc();
                let op = match range.kind {
                    BoundType::Inclusive => "..",
                    BoundType::Exclusive => ".<",
                };
                intersperse_with_space([BoxDoc::text(op), expr])
            }
            (Some(range), None) => {
                let expr = range.expr.to_doc();
                let op = match range.kind {
                    BoundType::Inclusive => "..",
                    BoundType::Exclusive => "<.",
                };
                intersperse_with_space([expr, BoxDoc::text(op)])
            }
            (Some(left), Some(right)) => {
                let op = match (left.kind, right.kind) {
                    (BoundType::Inclusive, BoundType::Inclusive) => "..",
                    (BoundType::Inclusive, BoundType::Exclusive) => ".<",
                    (BoundType::Exclusive, BoundType::Inclusive) => "<.",
                    (BoundType::Exclusive, BoundType::Exclusive) => "<<",
                };
                let left = left.expr.to_doc();
                let right = right.expr.to_doc();
                intersperse_with_space([left, BoxDoc::text(op), right])
            }
        };
        bracket("[", "]", range)
    }
}
impl<T: PrettyPrintType> TraverseType for Range<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.left
            .as_mut()
            .map(|bound| bound.traverse_type(data, for_type, for_scheme))
            .unwrap_or(Ok(()))?;
        self.right
            .as_mut()
            .map(|bound| bound.traverse_type(data, for_type, for_scheme))
            .unwrap_or(Ok(()))?;
        Ok(())
    }
}
