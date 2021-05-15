use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Pattern<'a> {
    Ignore,
    Var(&'a str),
    MutVar(&'a str),
    Struct(StructPattern<'a>),
    Array(Box<[Pattern<'a>]>),
    ArrayWithRest(ArrayWithRest<'a>),
    Tag(TaggedPattern<'a>),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ArrayWithRest<'a> {
    pub left: Box<[Pattern<'a>]>,
    pub rest: Box<Pattern<'a>>,
    pub right: Box<[Pattern<'a>]>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StructPattern<'a> {
    pub fields: HashMap<&'a str, Pattern<'a>>,
    pub rest: Option<Box<Pattern<'a>>>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TaggedPattern<'a> {
    pub tag: &'a str,
    pub pattern: Option<Box<Pattern<'a>>>,
}
