use crate::ast::expr::compound::Array;
use crate::ast::expr::compound::Struct;
use crate::ast::expr::control_flow::Block;
use crate::ast::expr::control_flow::Break;
use crate::ast::expr::control_flow::Continue;
use crate::ast::expr::control_flow::For;
use crate::ast::expr::control_flow::Fun;
use crate::ast::expr::control_flow::If;
use crate::ast::expr::control_flow::Return;
use crate::ast::expr::control_flow::While;
use crate::ast::expr::operator::BinaryExpr;
use crate::ast::expr::operator::Call;
use crate::ast::expr::operator::Property;
use crate::ast::expr::operator::Slice;
use crate::ast::expr::operator::UnaryExpr;
use crate::ast::expr::range::Range;

pub mod compound;
pub mod control_flow;
pub mod operator;
pub mod range;

#[derive(Debug, PartialEq, Clone)]
pub enum Expr<'a> {
    True,
    False,
    Null,

    Ident(&'a str),
    Char(u8),
    Str(Vec<u8>),
    UInt(u64),
    Float(f64),

    Break(Break<'a>),
    Continue(Continue<'a>),
    Return(Return<'a>),

    Unary(UnaryExpr<'a>),
    Binary(BinaryExpr<'a>),

    Array(Array<'a>),
    ArrayRange(Range<'a>),
    Struct(Struct<'a>),

    Property(Property<'a>),
    OptionalProperty(Property<'a>),
    Slice(Slice<'a>),
    OptionalSlice(Slice<'a>),
    Call(Call<'a>),

    Block(Block<'a>),
    Fun(Fun<'a>),
    If(If<'a>),
    For(For<'a>),
    While(While<'a>),
    Loop(Block<'a>),
}
