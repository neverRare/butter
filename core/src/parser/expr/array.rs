use crate::ast::expr::range::Bound;
use crate::ast::expr::range::Range;
use crate::parser::expr::expr;
use crate::parser::expr::infix_0;
use crate::parser::lex;
use combine::between;
use combine::optional;
use combine::parser::char::char;
use combine::parser::range::recognize;
use combine::satisfy;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

pub fn range<'a, I>() -> impl Parser<I, Output = Range<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let operator = || {
        recognize((
            satisfy(|ch: char| matches!(ch, '.' | '>')),
            satisfy(|ch: char| matches!(ch, '.' | '<')),
        ))
    };
    let range = || {
        (
            optional(expr(infix_0())),
            lex(operator()),
            optional(expr(infix_0())),
        )
            .map(|(left, op, right)| {
                let op: &str = op;
                let left = match (left, &op[..1]) {
                    (Some(expr), ".") => Bound::Inclusive(Box::new(expr)),
                    (Some(expr), ">") => Bound::Exclusive(Box::new(expr)),
                    (None, _) => Bound::NoBound,
                    _ => unreachable!(),
                };
                let right = match (&op[1..], right) {
                    (".", Some(expr)) => Bound::Inclusive(Box::new(expr)),
                    ("<", Some(expr)) => Bound::Exclusive(Box::new(expr)),
                    (_, None) => Bound::NoBound,
                    _ => unreachable!(),
                };
                Range { left, right }
            })
    };
    between(lex(char('[')), lex(char(']')), range())
}
