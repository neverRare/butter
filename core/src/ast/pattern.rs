#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Pattern<'a> {
    Ignore,
    Var(&'a str),
    Struct(StructPattern<'a>),
    Array(ArrayPattern<'a>),
    ArrayWithRest(ArrayWithRest<'a>),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ArrayPattern<'a> {
    pub elements: Vec<Pattern<'a>>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ArrayWithRest<'a> {
    pub left: Vec<Pattern<'a>>,
    pub rest: Box<Pattern<'a>>,
    pub right: Vec<Pattern<'a>>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StructPattern<'a> {
    pub fields: Vec<PatternField<'a>>,
    pub rest: Option<Box<Pattern<'a>>>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Param<'a> {
    pub fields: Vec<PatternField<'a>>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PatternField<'a> {
    pub name: &'a str,
    pub content: Pattern<'a>,
}
