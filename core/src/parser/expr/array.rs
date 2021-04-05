use crate::ast::expr::range::Bound;
use crate::ast::expr::range::Range;
use crate::parser::expr::expr;
use crate::parser::lex;
use combine::between;
use combine::choice;
use combine::optional;
use combine::parser::char::char;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

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
pub fn range<'a, I>() -> impl Parser<I, Output = Range<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
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
