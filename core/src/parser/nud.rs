use crate::parser::Array;
use crate::parser::op::UnaryOp;
use crate::parser::unpacking::Unpacking;
use crate::parser::Block;
use crate::parser::Expr;
use crate::parser::Struct;

enum Nud<'a> {
    // keyword literals
    True,
    False,
    Null,
    // literals
    Ident(&'a str),
    Str(Vec<u8>),
    Char(u8),
    Num(Num),
    // data literals
    Array(Array),
    Struct(Struct<'a>),
    // operators
    Group(Expr),
    Unary(UnaryOp, Box<Nud<'a>>),
    // control flow
    Block(Block),
    If(If),
    While(While),
    Loop(Block),
    For(For),
}
enum Num {
    UInt(u64),
    Float(f64),
}
struct If {
    condition: Expr,
    body: Block,
    otherwise: Box<Else>,
}
enum Else {
    None,
    Block(Block),
    If(If),
}
struct While {
    condition: Expr,
    body: Block,
}
struct For {
    var: Unpacking,
    val: Expr,
    body: Block,
}
