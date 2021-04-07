use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Pattern<'a> {
    Ignore,
    Var(&'a str),
    Struct(StructPattern<'a>),
    Array(Vec<Pattern<'a>>),
    ArrayWithRest(ArrayWithRest<'a>),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ArrayWithRest<'a> {
    pub left: Vec<Pattern<'a>>,
    pub rest: Box<Pattern<'a>>,
    pub right: Vec<Pattern<'a>>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StructPattern<'a> {
    pub fields: HashMap<&'a str, Pattern<'a>>,
    pub rest: Box<Pattern<'a>>,
}
