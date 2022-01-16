use crate::{expr::expr, lex, sep_optional_between, size_of};
use combine::{between, parser::char::char, ParseError, Parser, Stream};
use hir::expr::{Tuple, TupleWithSplat};

pub(crate) fn tuple<I, T>() -> impl Parser<I, Output = Tuple<T>>
where
    I: Stream<Token = char>,
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
pub(crate) fn print_tuple_sizes() {
    println!(
        "{}: {}",
        concat!(module_path!(), "::tuple"),
        size_of(&tuple::<&str, ()>()),
    );
}
