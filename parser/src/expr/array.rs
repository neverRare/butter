use crate::expr::expr;
use crate::lex;
use combine::between;
use combine::choice;
use combine::optional;
use combine::parser::char::char;
use combine::sep_end_by;
use combine::value;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;
use hir::expr::Bound;
use hir::expr::Element;
use hir::expr::ElementKind;
use hir::expr::Range;

pub fn range_operator<'a, I>() -> impl Parser<I, Output = (bool, bool)>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        choice((char('.').map(|_| true), (char('>').map(|_| false)))),
        choice((char('.').map(|_| true), (char('<').map(|_| false)))),
    )
}
pub fn range<'a, I, T>() -> impl Parser<I, Output = Range<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    let range = || {
        (optional(expr(0)), lex(range_operator()), optional(expr(0))).map(
            |(left, (left_in, right_in), right)| {
                let left = match (left, left_in) {
                    (Some(expr), true) => Bound::Inclusive(Box::new(expr)),
                    (Some(expr), false) => Bound::Exclusive(Box::new(expr)),
                    (None, _) => Bound::NoBound,
                };
                let right = match (right_in, right) {
                    (true, Some(expr)) => Bound::Inclusive(Box::new(expr)),
                    (false, Some(expr)) => Bound::Exclusive(Box::new(expr)),
                    (_, None) => Bound::NoBound,
                };
                Range { left, right }
            },
        )
    };
    between(lex(char('[')), lex(char(']')), range())
}
pub fn array<'a, I, T>() -> impl Parser<I, Output = Box<[Element<'a, T>]>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    let element_kind = || {
        choice((
            char('*').map(|_| ElementKind::Splat),
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
}
