use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Pattern<'a, T> {
    True,
    False,
    UInt(u64),
    Int(i64),
    Ignore,
    Var(Var<'a, T>),
    Record(RecordPattern<'a, T>),
    Tuple(ListPattern<'a, T>),
    Array(ListPattern<'a, T>),
    Tag(TaggedPattern<'a, T>),
    Ref(Box<Pattern<'a, T>>),
}
impl<'a, T> Pattern<'a, T> {
    pub fn field_name(&self) -> Option<&'a str> {
        match self {
            Self::Var(var) => Some(var.ident),
            _ => None,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Var<'a, T> {
    pub ident: &'a str,
    pub mutable: bool,
    pub bind_to_ref: bool,
    pub ty: T,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ListPattern<'a, T> {
    List(Box<[Pattern<'a, T>]>),
    ListWithRest(ListWithRest<'a, T>),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ListWithRest<'a, T> {
    pub left: Box<[Pattern<'a, T>]>,
    pub rest: Box<Pattern<'a, T>>,
    pub right: Box<[Pattern<'a, T>]>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RecordPattern<'a, T> {
    pub fields: HashMap<&'a str, Pattern<'a, T>>,
    pub rest: Option<Box<Pattern<'a, T>>>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TaggedPattern<'a, T> {
    pub tag: &'a str,
    pub pattern: Option<Box<Pattern<'a, T>>>,
}
