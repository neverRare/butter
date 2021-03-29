use combine::error::StreamError;
use combine::parser::range::take_while1;
use combine::stream::StreamErrorFor;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

pub fn ident_or_keyword<'a, I>() -> impl Parser<I, Output = &'a str>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    take_while1(|ch: char| ch.is_alphanumeric() || ch == '_')
}
pub fn keyword<'a, I>(keyword: &'static str) -> impl Parser<I, Output = &'a str>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    ident_or_keyword::<'a>().and_then(move |ident| {
        if ident == keyword {
            Ok(ident)
        } else {
            Err(<StreamErrorFor<I>>::expected_static_message(keyword))
        }
    })
}
pub fn ident<'a, I>() -> impl Parser<I, Output = &'a str>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    ident_or_keyword::<'a>().and_then(|ident| match ident {
        "_" | "break" | "clone" | "continue" | "else" | "false" | "for" | "if" | "in" | "loop"
        | "null" | "return" | "true" | "while" => {
            Err(<StreamErrorFor<I>>::unexpected_static_message("keyword"))
        }
        _ => Ok(ident),
    })
}
