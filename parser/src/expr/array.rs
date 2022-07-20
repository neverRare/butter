use crate::{expr::expr, lex};
use combine::{
    between, choice, optional, parser::char::char, sep_end_by, value, ParseError, Parser, Stream,
};
use hir::expr::{Bound, BoundType, Element, ElementKind, Range};

pub(crate) fn range_operator<I>() -> impl Parser<I, Output = (BoundType, BoundType)>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        choice((
            char('.').with(value(BoundType::Inclusive)),
            char('>').with(value(BoundType::Exclusive)),
        )),
        choice((
            char('.').with(value(BoundType::Inclusive)),
            char('<').with(value(BoundType::Exclusive)),
        )),
    )
        .expected("range operator")
}
pub(crate) fn range<I>() -> impl Parser<I, Output = Range<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let range = || {
        (optional(expr(0)), lex(range_operator()), optional(expr(0))).map(
            |(left, (left_in, right_in), right)| {
                let left = left.map(|left| Bound {
                    kind: left_in,
                    expr: Box::new(left),
                });
                let right = right.map(|right| Bound {
                    kind: right_in,
                    expr: Box::new(right),
                });
                Range { left, right }
            },
        )
    };
    between(lex(char('[')), lex(char(']')), range()).expected("range array")
}
pub(crate) fn array<I>() -> impl Parser<I, Output = Box<[Element<()>]>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let element_kind = || {
        choice((
            char('*').with(value(ElementKind::Splat)),
            value(ElementKind::Element),
        ))
    };
    let element = || (lex(element_kind()), expr(0)).map(|(kind, expr)| Element { expr, kind });
    between(
        lex(char('[')),
        lex(char(']')),
        sep_end_by(element(), lex(char(','))),
    )
    .map(Vec::into)
    .expected("array")
}
