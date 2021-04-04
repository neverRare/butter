use combine::eof;
use combine::error::StreamError;
use combine::look_ahead;
use combine::parser::char::string;
use combine::parser::range::recognize;
use combine::parser::range::take_while;
use combine::satisfy;
use combine::stream::StreamErrorFor;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

fn rest(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}
pub fn ident_or_keyword<'a, I>() -> impl Parser<I, Output = &'a str>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let start = move |ch: char| rest(ch) && !('0'..='9').contains(&ch);
    recognize((satisfy(start), take_while(rest)))
}
pub fn keyword<'a, I>(keyword: &'static str) -> impl Parser<I, Output = &'a str>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        string(keyword),
        look_ahead(satisfy(|ch| !rest(ch)).map(|_| ()).or(eof())),
    )
        .map(|(keyword, _)| keyword)
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
    use crate::parser::ident_keyword::ident;
    use crate::parser::ident_keyword::ident_or_keyword;
    use crate::parser::ident_keyword::keyword;
    use combine::EasyParser;

    #[test]
    fn test_keyword() {
        assert_eq!(keyword("true").easy_parse("true"), Ok(("true", "")));
    }
    #[test]
    fn non_keyword() {
        assert!(keyword("true").easy_parse("true_false").is_err());
    }
    #[test]
    fn test_ident_or_keyword() {
        assert_eq!(ident_or_keyword().easy_parse("foo"), Ok(("foo", "")));
    }
    #[test]
    fn non_ident() {
        assert!(ident().easy_parse("12").is_err());
    }
}
