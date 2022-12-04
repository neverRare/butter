use crate::{expr::expr, lex, sep_optional_between};
use combine::{between, parser::char::char, ParseError, Parser, Stream};
use hir::expr::{Collection, Expr, WithSplat};

pub(super) fn tuple<I>() -> impl Parser<I, Output = Collection<Expr<()>, ()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let fields = || {
        sep_optional_between(|| expr(0), lex(char('*')).with(expr(0)), || lex(char(','))).map(
            |(left, rest_right)| {
                let left: Vec<_> = left;
                match rest_right {
                    Some((rest, right)) => Collection::WithSplat(WithSplat {
                        left: left.into(),
                        splat: Box::new(rest),
                        right: right.into(),
                    }),
                    None => Collection::Collection(left.into()),
                }
            },
        )
    };
    between(lex(char('(')), lex(char(')')), fields()).expected("tuple")
}
