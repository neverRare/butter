use crate::{
    Atom, PrettyPrintType, TraverseType, bracket, intersperse_with_line, intersperse_with_space,
};
use pretty::BoxDoc;
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Pattern<T> {
    pub pattern: PatternKind<T>,
    pub ty: T,
}
impl<T> Pattern<T> {
    pub fn field_name(&self) -> Option<Atom> {
        self.pattern.field_name()
    }
}
impl<T: PrettyPrintType> Pattern<T> {
    pub fn to_doc(&self) -> BoxDoc where {
        let pattern = self.pattern.to_doc();
        match self.ty.to_doc() {
            Some(ty) => intersperse_with_space([pattern, BoxDoc::text(":"), ty]),
            None => pattern,
        }
    }
}
impl<T: PrettyPrintType> TraverseType for Pattern<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut T, &U) -> Result<(), E>,
        for_scheme: fn(&mut T::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        for_type(&mut self.ty, data)?;
        self.pattern.traverse_type(data, for_type, for_scheme)?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PatternKind<T> {
    UInt(u64),
    Int(i64),
    Discard,
    Var(Var),
    Record(RecordPattern<T>),
    Tuple(ListPattern<T>),
    Param(Box<[TypedVar<T>]>),
    Array(ListPattern<T>),
    Tag(TaggedPattern<T>),
    Ref(Box<Pattern<T>>),
}
impl<T: PrettyPrintType> PatternKind<T> {
    pub fn to_doc(&self) -> BoxDoc {
        match self {
            Self::UInt(uint) => BoxDoc::as_string(uint),
            Self::Int(int) => BoxDoc::as_string(int),
            Self::Discard => BoxDoc::text("_"),
            Self::Var(var) => var.to_doc(),
            Self::Record(record) => {
                let fields =
                    intersperse_with_line(
                        record
                            .fields
                            .iter()
                            .map(|(key, pattern)| {
                                intersperse_with_space([
                                    BoxDoc::text(key as &str),
                                    BoxDoc::text("="),
                                    pattern.to_doc(),
                                ])
                            })
                            .chain(record.rest.iter().map(|pattern| {
                                BoxDoc::concat([BoxDoc::text("*"), pattern.to_doc()])
                            }))
                            .map(|pattern| BoxDoc::concat([pattern, BoxDoc::text(",")])),
                    );
                bracket("(", ")", fields)
            }
            Self::Tuple(tuple) => bracket("(", ")", tuple.to_doc()),
            Self::Param(param) => {
                let iter = param
                    .iter()
                    .map(TypedVar::to_doc)
                    .map(|var| BoxDoc::concat([var, BoxDoc::text(",")]));
                bracket("(", ")", intersperse_with_line(iter))
            }
            Self::Array(arr) => bracket("(", ")", arr.to_doc()),
            Self::Tag(tag) => tag.to_doc(),
            Self::Ref(pattern) => BoxDoc::concat([BoxDoc::text("&"), pattern.to_doc()]),
        }
    }
}
impl<T: PrettyPrintType> TraverseType for PatternKind<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut T, &U) -> Result<(), E>,
        for_scheme: fn(&mut T::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        match self {
            PatternKind::UInt(_) => (),
            PatternKind::Int(_) => (),
            PatternKind::Discard => (),
            PatternKind::Var(_) => (),
            PatternKind::Record(record) => record.traverse_type(data, for_type, for_scheme)?,
            PatternKind::Tuple(tuple) => tuple.traverse_type(data, for_type, for_scheme)?,
            PatternKind::Param(param) => {
                for var in param.iter_mut() {
                    var.traverse_type(data, for_type, for_scheme)?
                }
            }
            PatternKind::Array(array) => array.traverse_type(data, for_type, for_scheme)?,
            PatternKind::Tag(tag) => tag.traverse_type(data, for_type, for_scheme)?,
            PatternKind::Ref(reference) => reference.traverse_type(data, for_type, for_scheme)?,
        }
        Ok(())
    }
}
impl<T> PatternKind<T> {
    pub fn field_name(&self) -> Option<Atom> {
        match self {
            Self::Var(var) => Some(var.ident.clone()),
            _ => None,
        }
    }
}
impl PatternKind<()> {
    pub fn into_untyped(self) -> Pattern<()> {
        Pattern {
            pattern: self,
            ty: (),
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Var {
    pub ident: Atom,
    pub mutable: bool,
    pub bind_to_ref: bool,
}
impl Var {
    pub fn to_doc(&self) -> BoxDoc {
        let mutable = if self.mutable {
            BoxDoc::text("mut")
        } else {
            BoxDoc::nil()
        };
        let bind_to_ref = if self.mutable {
            BoxDoc::text("&<")
        } else {
            BoxDoc::nil()
        };
        let ident = BoxDoc::text(&self.ident as &str);
        intersperse_with_space([mutable, BoxDoc::concat([bind_to_ref, ident])])
    }
}
impl Var {
    pub fn into_untyped(self) -> TypedVar<()> {
        TypedVar { var: self, ty: () }
    }
}
impl Display for Var {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        let mutable = if self.mutable { "mut " } else { "" };
        let bind_to_ref = if self.bind_to_ref { "&<" } else { "" };
        write!(fmt, "{mutable}{bind_to_ref}{}", self.ident)?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypedVar<T> {
    pub var: Var,
    pub ty: T,
}
impl<T: PrettyPrintType> TypedVar<T> {
    pub fn to_doc(&self) -> BoxDoc {
        let var = self.var.to_doc();
        match self.ty.to_doc() {
            Some(ty) => intersperse_with_space([var, BoxDoc::text(":"), ty]),
            None => var,
        }
    }
}
impl<T: PrettyPrintType> TraverseType for TypedVar<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut T, &U) -> Result<(), E>,
        _for_scheme: fn(&mut T::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        for_type(&mut self.ty, data)?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ListPattern<T> {
    List(Box<[Pattern<T>]>),
    ListWithRest(ListWithRest<T>),
}
impl<T: PrettyPrintType> TraverseType for ListPattern<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        match self {
            ListPattern::List(list) => {
                for pattern in list.iter_mut() {
                    pattern.traverse_type(data, for_type, for_scheme)?;
                }
            }
            ListPattern::ListWithRest(list) => list.traverse_type(data, for_type, for_scheme)?,
        }
        Ok(())
    }
}
impl<T: PrettyPrintType> ListPattern<T> {
    pub fn to_doc(&self) -> BoxDoc {
        match self {
            ListPattern::List(list) => {
                let iter = list
                    .iter()
                    .map(Pattern::to_doc)
                    .map(|pattern| BoxDoc::concat([pattern, BoxDoc::text(",")]));
                intersperse_with_line(iter)
            }
            ListPattern::ListWithRest(list) => {
                let iter = list
                    .left
                    .iter()
                    .map(Pattern::to_doc)
                    .chain([BoxDoc::concat([BoxDoc::text("*"), list.rest.to_doc()])])
                    .chain(list.right.iter().map(Pattern::to_doc))
                    .map(|pattern| BoxDoc::concat([pattern, BoxDoc::text(",")]));
                intersperse_with_line(iter)
            }
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ListWithRest<T> {
    pub left: Box<[Pattern<T>]>,
    pub rest: Box<Pattern<T>>,
    pub right: Box<[Pattern<T>]>,
}
impl<T: PrettyPrintType> TraverseType for ListWithRest<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        for pattern in self.left.iter_mut() {
            pattern.traverse_type(data, for_type, for_scheme)?;
        }
        self.rest.traverse_type(data, for_type, for_scheme)?;
        for pattern in self.right.iter_mut() {
            pattern.traverse_type(data, for_type, for_scheme)?;
        }
        Ok(())
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RecordPattern<T> {
    pub fields: HashMap<Atom, Pattern<T>>,
    pub rest: Option<Box<Pattern<T>>>,
}
impl<T: PrettyPrintType> TraverseType for RecordPattern<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        for (_, pattern) in self.fields.iter_mut() {
            pattern.traverse_type(data, for_type, for_scheme)?;
        }
        self.rest
            .as_mut()
            .map(|pattern| pattern.traverse_type(data, for_type, for_scheme))
            .unwrap_or(Ok(()))?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TaggedPattern<T> {
    pub tag: Atom,
    pub pattern: Option<Box<Pattern<T>>>,
}
impl<T: PrettyPrintType> TaggedPattern<T> {
    pub fn to_doc(&self) -> BoxDoc {
        let pattern = match &self.pattern {
            Some(pattern) => {
                let expr = pattern.to_doc();
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
            pattern,
        ])
    }
}
impl<T: PrettyPrintType> TraverseType for TaggedPattern<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.pattern
            .as_mut()
            .map(|pattern| pattern.traverse_type(data, for_type, for_scheme))
            .unwrap_or(Ok(()))
    }
}
