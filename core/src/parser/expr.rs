use crate::parser::ident_keyword::ident;
use crate::ast::expr::Expr;
use crate::parser::ident_keyword::keyword;
use crate::parser::lex;
use crate::parser::Parser;
use combine::attempt;
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
    between(lex(char('(')), lex(char(')')), expr(0))
}
fn prefix_expr<'a, I>() -> impl Parser<I, Output = Expr<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        attempt(lex(ident())).map(Expr::Var),
        lex(keyword("false")).map(|_| Expr::False),
        lex(keyword("null")).map(|_| Expr::Null),
        lex(keyword("true")).map(|_| Expr::True),
        group(),
    ))
}
fn expr_<'a, I>(precedence: u32) -> impl Parser<I, Output = Expr<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    prefix_expr()
}
parser! {
    pub fn expr['a, I](precedence: u32)(I) -> Expr<'a>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        expr_(*precedence)
    }
}
