#[derive(Clone, Copy)]
pub enum Num {
    UInt(u64),
    Float(f64),
}
#[derive(Clone, Copy)]
pub enum UnaryOp {
    Plus,
    Minus,
    Ref,
    Not,
    Clone,
}
#[derive(Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mult,
    Div,
    FlrDiv,
    Mod,
    And,
    Or,
    LazyAnd,
    LazyOr,
    Eq,
    NotEq,
    Gt,
    Gte,
    Lt,
    Lte,
    Concat,
    NullOr,
}
#[derive(Clone, Copy)]
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
    Num(Num),
    Path,

    Break,
    Continue,
    Return,

    Unary(UnaryOp),
    Binary(BinaryOp),

    Declare,
    FunDeclare,
    Assign,

    Array,
    Struct,

    Property,
    OptionalProperty,
    Index,
    OptionalIndex,

    Block(bool),
    Fun,
    If,
    Else,
    For,
    While,
    Loop,
}
impl NodeType {
    pub fn place(self) -> bool {
        match self {
            Self::Ident => true,
            Self::Property => true,
            Self::Index => true,
            Self::Unary(UnaryOp::Ref) => true,
            _ => false,
        }
    }
}