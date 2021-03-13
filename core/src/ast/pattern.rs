pub enum PatternType<'a> {
    Ignore,
    Ident(&'a str),
    Struct(StructPattern<'a>),
    Array(ArrayPattern<'a>),
    ArrayWithRest(ArrayWithRest<'a>),
}
pub struct Pattern<'a> {
    pub span: &'a str,
    pub unpack: PatternType<'a>,
}
pub struct ArrayPattern<'a> {
    pub elements: Vec<Pattern<'a>>,
}
pub struct ArrayWithRest<'a> {
    pub left: Vec<Pattern<'a>>,
    pub rest: Box<Pattern<'a>>,
    pub right: Vec<Pattern<'a>>,
}
pub struct StructPattern<'a> {
    pub fields: Vec<PatternField<'a>>,
    pub rest: Option<Box<Pattern<'a>>>,
}
pub struct Param<'a> {
    pub fields: Vec<PatternField<'a>>,
}
pub struct PatternField<'a> {
    pub name: &'a str,
    pub content: Pattern<'a>,
}
