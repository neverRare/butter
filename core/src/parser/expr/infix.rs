use crate::ast::expr::compound::Arg;
use crate::ast::expr::range::Range;
use crate::parser::expr::Expr;
use crate::parser::ident_keyword::ident;
use crate::parser::lex;
use combine::choice;
use combine::parser::char::char;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

pub enum PartialAst<'a> {
    Add(Expr<'a>),
    Sub(Expr<'a>),
    Multiply(Expr<'a>),
    Div(Expr<'a>),
    FloorDiv(Expr<'a>),
    Mod(Expr<'a>),
    And(Expr<'a>),
    Or(Expr<'a>),
    LazyAnd(Expr<'a>),
    LazyOr(Expr<'a>),
    Equal(Expr<'a>),
    NotEqual(Expr<'a>),
    Greater(Expr<'a>),
    GreaterEqual(Expr<'a>),
    Less(Expr<'a>),
    LessEqual(Expr<'a>),
    Concatenate(Expr<'a>),
    NullOr(Expr<'a>),

    Assign(Expr<'a>),

    Property(&'a str),
    OptionalProperty(&'a str),
    Index(Expr<'a>),
    OptionalIndex(Expr<'a>),
    Slice(Range<'a>),
    OptionalSlice(Range<'a>),
    Call(Arg<'a>),
}
pub fn infix_9<'a, I>() -> impl Parser<I, Output = PartialAst<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice(((lex(char('.')), ident()).map(|(_, name)| PartialAst::Property(name)),))
}
