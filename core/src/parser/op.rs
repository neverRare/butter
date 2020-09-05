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