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
    Array(Box<[Pattern<'a, T>]>),
    ArrayWithRest(ArrayWithRest<'a, T>),
    Tag(TaggedPattern<'a, T>),
    Ref(Box<Pattern<'a, T>>),
    RefMut(Box<Pattern<'a, T>>),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Var<'a, T> {
    pub ident: &'a str,
    pub mutable: bool,
    pub bind_to_ref: bool,
    pub ty: T,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ArrayWithRest<'a, T> {
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
