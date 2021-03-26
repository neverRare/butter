use crate::ast::pattern::ArrayWithRest;
use crate::ast::pattern::Pattern;
use crate::parser_2::ident_keyword::ident;
use crate::parser_2::ident_keyword::keyword;
use crate::parser_2::lex;
use combine::attempt;
use combine::choice;
use combine::optional;
use combine::parser;
use combine::parser::char::char;
use combine::sep_by;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

fn elements<'a, I>() -> impl Parser<I, Output = Vec<Pattern<'a>>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    sep_by(pattern(), lex(char(',')))
}
fn rest<'a, I>() -> impl Parser<I, Output = Pattern<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (lex(char('*')), pattern()).map(|(_, pattern)| pattern)
}
fn array<'a, I>() -> impl Parser<I, Output = Pattern<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        lex(char('[')),
        elements(),
        optional((rest(), optional((lex(char(',')), elements())))),
        lex(char(']')),
    )
        .map(|(_, left, rest_right, _)| match rest_right {
            Some((rest, right)) => {
                let right = match right {
                    Some((_, right)) => right,
                    None => vec![],
                };
                Pattern::ArrayWithRest(ArrayWithRest {
                    left,
                    rest: Box::new(rest),
                    right,
                })
            }
            None => Pattern::Array(left),
        })
}
fn pattern_<'a, I>() -> impl Parser<I, Output = Pattern<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        array(),
        attempt(lex(keyword("_"))).map(|_| Pattern::Ignore),
        lex(ident()).map(|ident| Pattern::Var(ident)),
    ))
}
parser! {
    pub fn pattern['a, I]()(I) -> Pattern<'a>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        pattern_()
    }
}
