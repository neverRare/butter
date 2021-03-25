use combine::parser::char::spaces;
use combine::parser::char::string;
use combine::parser::range::take_while;
use combine::skip_many;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

mod ident_keyword;

fn comments<'a, I>() -> impl Parser<I, Output = ()>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    skip_many((string("--"), take_while(|ch: char| ch != '\n')))
}
fn insignificants<'a, I>() -> impl Parser<I, Output = ()>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    skip_many(spaces().or(comments()))
}
fn lex<'a, I, P>(parser: P) -> impl Parser<I, Output = P::Output>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    P: Parser<I>
{
    parser.skip(insignificants())
}
