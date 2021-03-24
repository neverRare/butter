use combine::parser::char::spaces;
use combine::parser::char::string;
use combine::parser::range::take_while;
use combine::skip_many;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

mod ident_keyword;

pub fn comments<'a, I>() -> impl Parser<I, Output = ()>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    skip_many((string("--"), take_while(|ch: char| ch != '\n')))
}
pub fn insignificants<'a, I>() -> impl Parser<I, Output = ()>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    skip_many(spaces().or(comments()))
}
