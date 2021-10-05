use crate::{expr::expr, lex, sep_optional_between};
use combine::{between, parser::char::char, ParseError, Parser, RangeStream};
use hir::expr::{Tuple, TupleWithSplat};

pub(crate) fn tuple<'a, I, T>() -> impl Parser<I, Output = Tuple<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    let fields = || {
        sep_optional_between(|| expr(0), lex(char('*')).with(expr(0)), || lex(char(','))).map(
            |(left, rest_right)| {
                let left: Vec<_> = left;
                match rest_right {
                    Some((rest, right)) => Tuple::TupleWithSplat(TupleWithSplat {
                        left: left.into(),
                        splat: Box::new(rest),
                        right: right.into(),
                    }),
                    None => Tuple::Tuple(left.into()),
                }
            },
        )
    };
    between(lex(char('(')), lex(char(')')), fields()).expected("tuple")
}
