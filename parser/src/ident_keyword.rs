use combine::{
    error::StreamError,
    not_followed_by,
    parser::{char::string, combinator::recognize},
    satisfy, skip_many,
    stream::StreamErrorFor,
    ParseError, Parser, Stream,
};
use hir::{keyword, Atom};

fn rest(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}
pub(crate) fn ident_or_keyword<I>() -> impl Parser<I, Output = Atom>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let start = move |ch: char| rest(ch) && !('0'..='9').contains(&ch);
    recognize::<String, _, _>((satisfy(start), skip_many(satisfy(rest))))
        .map(Atom::from)
        .expected("identifier")
}
pub(crate) fn keyword<I>(keyword: &'static str) -> impl Parser<I, Output = ()>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    string(keyword)
        .skip(not_followed_by(satisfy(rest)))
        .map(|_| ())
}
pub(crate) fn ident<I>() -> impl Parser<I, Output = Atom>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    ident_or_keyword().and_then(|ident| match ident {
        keyword!("_")
        | keyword!("break")
        | keyword!("clone")
        | keyword!("continue")
        | keyword!("else")
        | keyword!("false")
        | keyword!("for")
        | keyword!("if")
        | keyword!("in")
        | keyword!("loop")
        | keyword!("match")
        | keyword!("mut")
        | keyword!("ref")
        | keyword!("return")
        | keyword!("true")
        | keyword!("while") => Err(<StreamErrorFor<I>>::unexpected_static_message("keyword")),
        ident => Ok(ident),
    })
}
#[cfg(test)]
mod test {
    use crate::ident_keyword::{ident, ident_or_keyword, keyword};
    use combine::EasyParser;
    use hir::Atom;

    #[test]
    fn test_keyword() {
        assert_eq!(keyword("true").easy_parse("true"), Ok(((), "")));
    }
    #[test]
    fn non_keyword() {
        assert!(keyword("true").easy_parse("true_false").is_err());
    }
    #[test]
    fn test_ident_or_keyword() {
        assert_eq!(
            ident_or_keyword().easy_parse("foo"),
            Ok((Atom::from("foo"), ""))
        );
    }
    #[test]
    fn non_ident() {
        assert!(ident().easy_parse("12").is_err());
    }
}
