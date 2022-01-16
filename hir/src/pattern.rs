use std::collections::HashMap;
use string_cache::DefaultAtom;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Pattern<T> {
    True,
    False,
    UInt(u64),
    Int(i64),
    Ignore,
    Var(Var<T>),
    Record(RecordPattern<T>),
    Tuple(ListPattern<T>),
    Array(ListPattern<T>),
    Tag(TaggedPattern<T>),
    Ref(Box<Pattern<T>>),
}
impl<T> Pattern<T> {
    pub fn field_name(&self) -> Option<DefaultAtom> {
        match self {
            Self::Var(var) => Some(var.ident.clone()),
            _ => None,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Var<T> {
    pub ident: DefaultAtom,
    pub mutable: bool,
    pub bind_to_ref: bool,
    pub ty: T,
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
    pub fields: HashMap<DefaultAtom, Pattern<T>>,
    pub rest: Option<Box<Pattern<T>>>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TaggedPattern<T> {
    pub tag: DefaultAtom,
    pub pattern: Option<Box<Pattern<T>>>,
}
