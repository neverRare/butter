#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![forbid(unsafe_code)]

use combine::{
    attempt, eof, many, optional,
    parser::{
        char::{space, string},
        range::take_while,
    },
    skip_many, skip_many1, ParseError, Parser, RangeStream,
};
use hir::{expr::Expr, statement::Statement};

mod expr;
mod ident_keyword;
mod pattern;
mod statement;

combine::parser! {
    pub fn ast['a, I, T]()(I) -> Vec<Statement<'a, T>>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        T: Default,
    ] {
        optional(attempt(string("#!")).with(take_while(|ch| ch != '\n')))
            .with(insignificants())
            .with(many(statement::statement()))
            .skip(eof())
    }
}
pub fn expr_parser<'a, I, T>() -> impl Parser<I, Output = Expr<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    insignificants().with(expr::expr(0)).skip(eof())
}
fn comments<'a, I>() -> impl Parser<I, Output = ()>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    skip_many1((attempt(string("--")), take_while(|ch: char| ch != '\n')).expected("comment"))
}
pub fn insignificants<'a, I>() -> impl Parser<I, Output = ()>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    skip_many(skip_many1(space()).or(comments())).silent()
}
fn lex<'a, I, P>(parser: P) -> impl Parser<I, Output = P::Output>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    P: Parser<I>,
{
    parser.skip(insignificants())
}
#[cfg(test)]
mod test {
    use crate::insignificants;
    use combine::Parser;

    #[test]
    fn insignificant() {
        assert_eq!(
            insignificants()
                .parse("  -- comment\n  -- more comment")
                .unwrap(),
            ((), "")
        )
    }
}
