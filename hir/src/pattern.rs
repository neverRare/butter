use crate::Atom;
use std::collections::HashMap;

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
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PatternKind<T> {
    True,
    False,
    UInt(u64),
    Int(i64),
    Ignore,
    Var(Var),
    Record(RecordPattern<T>),
    Tuple(ListPattern<T>),
    Param(Box<[Pattern<T>]>),
    Array(ListPattern<T>),
    Tag(TaggedPattern<T>),
    Ref(Box<Pattern<T>>),
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ListPattern<T> {
    List(Box<[Pattern<T>]>),
    ListWithRest(ListWithRest<T>),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ListWithRest<T> {
    pub left: Box<[Pattern<T>]>,
    pub rest: Box<Pattern<T>>,
    pub right: Box<[Pattern<T>]>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RecordPattern<T> {
    pub fields: HashMap<Atom, Pattern<T>>,
    pub rest: Option<Box<Pattern<T>>>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TaggedPattern<T> {
    pub tag: Atom,
    pub pattern: Option<Box<Pattern<T>>>,
}
