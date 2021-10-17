#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![forbid(unsafe_code)]

use combine::{
    attempt, choice, eof, many, optional,
    parser::{
        char::{space, string},
        range::take_while,
    },
    sep_end_by, skip_many, skip_many1, ParseError, Parser, RangeStream,
};
use expr::print_expr_sizes;
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
combine::parser! {
    pub fn expr_parser['a, I, T]()(I) -> Expr<'a, T>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        T: Default,
    ] {
        insignificants().with(expr::expr(0)).skip(eof())
    }
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
fn sep_optional_between<'a, I, EP, RP, SP, C>(
    element: fn() -> EP,
    rest: RP,
    sep: fn() -> SP,
) -> impl Parser<I, Output = (C, Option<(RP::Output, C)>)>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    EP: Parser<I>,
    RP: Parser<I>,
    SP: Parser<I>,
    C: Extend<EP::Output> + Default,
{
    let no_rest = move || sep_end_by(element(), sep());
    let have_rest = move || {
        (
            attempt((many(element().skip(sep())), rest)),
            optional(sep().with(no_rest())).map(|right| right.unwrap_or_else(Default::default)),
        )
    };
    choice((
        have_rest().map(|((left, rest), right)| (left, Some((rest, right)))),
        no_rest().map(|collection| (collection, None)),
    ))
}
fn size_of<T>(_: &T) -> usize {
    std::mem::size_of::<T>()
}
pub fn print_sizes() {
    print_expr_sizes();
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
