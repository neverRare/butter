#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unary {
    Plus,
    Minus,
    Ref,
    Not,
    Clone,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Binary {
    Add,
    Sub,
    Multiply,
    Div,
    FloorDiv,
    Mod,
    And,
    Or,
    LazyAnd,
    LazyOr,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Concatenate,
    NullOr,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RangeType {
    Full,
    Inclusive,
    Exclusive,
    InclusiveExclusive,
    ExclusiveInclusive,
    FromInclusive,
    FromExclusive,
    ToInclusive,
    ToExclusive,
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NodeType {
    SplatOrRest,
    Label,

    CharInside(u8),

    True,
    False,
    Null,
    Ident,
    Char,
    Str,
    UInt(u64),
    Float(f64),

    Break,
    BreakWithExpr,
    Continue,
    Return,

    Unary(Unary),
    Binary(Binary),

    Declare,
    FunDeclare,
    Assign,

    Array,
    ArrayRange(RangeType),
    Struct,

    Property,
    OptionalProperty,
    Index,
    OptionalIndex,
    Slice(RangeType),
    OptionalSlice(RangeType),

    Block,
    BlockWithExpr,
    Fun,
    If,
    Else,
    For,
    While,
    Loop,
}
impl NodeType {
    pub fn place(self) -> bool {
        matches!(
            self,
            Self::Ident | Self::Property | Self::Index | Self::Unary(Unary::Ref)
        )
    }
}
