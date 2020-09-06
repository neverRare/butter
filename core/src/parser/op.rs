#[derive(Clone, Copy)]
pub enum UnaryOp {
    Plus,
    Minus,
    Ref,
    OptionalRef,
    Deref,
    OptionalDeref,
    Not,
    NotType(NotType),
}
#[derive(Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mult,
    Div,
    FlrDiv,
    Mod,
    Shl,
    Shr,
    BitAnd,
    BitOr,
    BitXor,
    And,
    Or,
    Eq,
    NotEq,
    Gt,
    Gte,
    Lt,
    Lte,
}
#[derive(Clone, Copy)]
pub enum NotType {
    U8,
    U16,
    U32,
    U64,
    I,
    I8,
    I16,
    I32,
    I64,
}
