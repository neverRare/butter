#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![forbid(unsafe_code)]

use combine::{
    attempt, choice, eof, many, none_of, optional,
    parser::char::{space, string},
    sep_end_by, skip_many, skip_many1, ParseError, Stream,
};
use hir::{expr::Expr, statement::Statement};

pub use combine::{EasyParser, Parser};

mod expr;
mod ident_keyword;
mod pattern;
mod statement;

combine::parser! {
    pub fn ast[T, I]()(I) -> Vec<Statement<T>>
    where [
        I: Stream<Token = char>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        T: Default,
    ] {
        optional(attempt(string("#!")).with(skip_many(none_of(['\n']))))
            .with(insignificants())
            .with(many(statement::statement()))
            .skip(eof())
    }
}
combine::parser! {
    pub fn expr_parser[T, I]()(I) -> Expr<T>
    where [
        I: Stream<Token = char>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        T: Default,
    ] {
        insignificants().with(expr::expr(0)).skip(eof())
    }
}
fn comments<I>() -> impl Parser<I, Output = ()>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    skip_many1((attempt(string("--")), skip_many(none_of(['\n']))))
}
pub fn insignificants<I>() -> impl Parser<I, Output = ()>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    skip_many(skip_many1(space()).or(comments())).silent()
}
fn lex<I, P>(parser: P) -> impl Parser<I, Output = P::Output>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    P: Parser<I>,
{
    parser.skip(insignificants())
}
fn sep_optional_between<I, EP, RP, SP, C>(
    element: fn() -> EP,
    rest: RP,
    sep: fn() -> SP,
) -> impl Parser<I, Output = (C, Option<(RP::Output, C)>)>
where
    I: Stream<Token = char>,
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
            optional(sep().with(no_rest())).map(|right| right.unwrap_or_default()),
        )
    };
    choice((
        have_rest().map(|((left, rest), right)| (left, Some((rest, right)))),
        no_rest().map(|collection| (collection, None)),
    ))
}
#[cfg(test)]
mod test {
    use crate::insignificants;
    use combine::Parser;
    use hir::expr::{Expr, PlaceExpr};
    use string_cache::DefaultAtom;

    pub(super) fn var_expr(var: &str) -> Expr<()> {
        Expr::Place(var_place(var))
    }
    pub(super) fn var_place(var: &str) -> PlaceExpr<()> {
        PlaceExpr::Var(DefaultAtom::from(var))
    }
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
