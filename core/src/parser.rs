use op::BinaryOp;
use op::UnaryOp;

mod num;
mod op;
mod string;

#[derive(Clone, Copy)]
pub enum Num {
    UInt(u64),
    Float(f64),
}
#[derive(Clone, Copy)]
enum Node {
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

    Abort,
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

    Error,
}
