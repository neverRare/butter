use combine::parser::char::string;
use combine::parser::range::take_while;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

mod ident_keyword;

pub fn comment<'a, I>() -> impl Parser<I, Output = &'a str>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (string("--"), take_while(|ch: char| ch != '\n'))
        .map(|(_, content): (&str, &str)| content.trim())
}
