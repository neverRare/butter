use crate::ast::expr::Expr;
use crate::parser::lex;
use crate::parser::Parser;
use combine::between;
use combine::choice;
use combine::parser;
use combine::parser::char::char;
use combine::ParseError;
use combine::RangeStream;

fn group<'a, I>() -> impl Parser<I, Output = Expr<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    between(lex(char('(')), lex(char(')')), prefix_expr())
}
fn prefix_expr_<'a, I>() -> impl Parser<I, Output = Expr<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((group(),))
}
parser! {
    pub fn prefix_expr['a, I]()(I) -> Expr<'a>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        prefix_expr_()
    }
}
