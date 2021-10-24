use combine::{
    error::StreamError,
    not_followed_by,
    parser::{
        char::string,
        range::{recognize, take_while},
    },
    satisfy,
    stream::StreamErrorFor,
    ParseError, Parser, RangeStream,
};

static KEYWORDS: [&str; 17] = [
    "_", "break", "clone", "continue", "else", "false", "for", "if", "in", "loop", "match", "mut",
    "ref", "return", "true", "void", "while",
];
fn rest(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}
pub(crate) fn ident_or_keyword<'a, I>() -> impl Parser<I, Output = &'a str>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let start = move |ch: char| rest(ch) && !('0'..='9').contains(&ch);
    recognize((satisfy(start), take_while(rest))).expected("identifier")
}
pub(crate) fn keyword<'a, I>(keyword: &'static str) -> impl Parser<I, Output = &'a str>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    debug_assert!(KEYWORDS.into_iter().any(|it| keyword == it));
    string(keyword).skip(not_followed_by(satisfy(rest)))
}
pub(crate) fn ident<'a, I>() -> impl Parser<I, Output = &'a str>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    ident_or_keyword::<'a>().and_then(|ident| {
        if KEYWORDS.into_iter().any(|it| ident == it) {
            Err(<StreamErrorFor<I>>::unexpected_static_message("keyword"))
        } else {
            Ok(ident)
        }
    })
}
#[cfg(test)]
mod test {
    use crate::ident_keyword::{ident, ident_or_keyword, keyword};
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
