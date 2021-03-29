use combine::error::StreamError;
use combine::parser::range::recognize;
use combine::parser::range::take_while;
use combine::satisfy;
use combine::stream::StreamErrorFor;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

pub fn ident_or_keyword<'a, I>() -> impl Parser<I, Output = &'a str>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let rest = |ch: char| ch.is_alphanumeric() || ch == '_';
    let start = move |ch: char| rest(ch) && !('0'..='9').contains(&ch);
    recognize((satisfy(start), take_while(rest)))
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
#[cfg(test)]
mod test {
    use crate::parser::ident_keyword::ident_or_keyword;
    use combine::EasyParser;

    #[test]
    fn ident() {
        assert_eq!(ident_or_keyword().easy_parse("foo"), Ok(("foo", "")))
    }
    #[test]
    fn non_ident() {
        assert!(ident_or_keyword().easy_parse("12").is_err())
    }
}
