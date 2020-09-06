use op::BinaryOp;
use op::UnaryOp;

mod op;

#[derive(Clone, Copy)]
pub enum Num {
    UInt(u64),
    Float(f64),
}
#[derive(Clone, Copy)]
enum NodeKind {
    Splat,
    Rest,
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

    Clone,
    Abort,
    Break,
    Continue,

    Unary(UnaryOp),
    Binary(BinaryOp),

    Declare,
    Assign,

    Array,
    Struct,

    Block(bool),
    Fun(bool),
    If,
    Else,
    For,
    While,
    Loop,
}
#[derive(Clone, Copy)]
struct Node<'a> {
    span: &'a str,
    kind: NodeKind,
    group: usize,
}
pub struct BigAst<'a>(Vec<Node<'a>>);
