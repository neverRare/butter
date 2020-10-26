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
}
