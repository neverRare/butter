use crate::ast::expr::range::Bound;
use crate::ast::expr::range::Range;
use crate::parser::expr::expr;
use crate::parser::expr::infix_0;
use crate::parser::lex;
use combine::attempt;
use combine::between;
use combine::choice;
use combine::optional;
use combine::parser::char::char;
use combine::parser::char::string;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

pub fn range<'a, I>() -> impl Parser<I, Output = Range<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let operator = || {
        lex(choice((
            attempt(string("..")),
            string(".<"),
            attempt(string(">.")),
            string("><"),
        )))
    };
    let range = || {
        (
            optional(expr(infix_0())),
            operator(),
            optional(expr(infix_0())),
        )
            .map(|(left, op, right)| {
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
