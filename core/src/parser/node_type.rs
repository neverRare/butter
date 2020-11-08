#[derive(Clone, Copy)]
pub enum Num {
    UInt(u64),
    Float(f64),
}
#[derive(Clone, Copy)]
pub enum Unary {
    Plus,
    Minus,
    Ref,
    Not,
    Clone,
}
#[derive(Clone, Copy)]
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

    Unary(Unary),
    Binary(Binary),

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
            Self::Unary(Unary::Ref) => true,
            _ => false,
        }
    }
    pub fn expr(self) -> bool {
        !matches!(
            self,
            Self::SplatOrRest
                | Self::Label
                | Self::CharInside(_)
                | Self::Declare
                | Self::FunDeclare
                | Self::Else
                | Self::For
                | Self::While
        )
    }
}
