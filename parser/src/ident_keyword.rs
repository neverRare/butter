use combine::{
    ParseError, Parser, Stream,
    error::StreamError,
    not_followed_by,
    parser::{char::string, combinator::recognize},
    satisfy, skip_many,
    stream::StreamErrorFor,
    value,
};
use hir::{Atom, keyword};

fn rest(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}
pub(super) fn ident_or_keyword<I>() -> impl Parser<I, Output = Atom>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let start = move |ch: char| rest(ch) && !ch.is_ascii_digit();
    recognize::<String, _, _>((satisfy(start), skip_many(satisfy(rest))))
        .map(Atom::from)
        .expected("identifier")
}
pub(super) fn keyword<I>(keyword: &'static str) -> impl Parser<I, Output = ()>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    string(keyword)
        .skip(not_followed_by(satisfy(rest)))
        .with(value(()))
}
pub(super) fn ident<I>() -> impl Parser<I, Output = Atom>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    ident_or_keyword().and_then(|ident| match ident {
        keyword!("_")
        | keyword!("alias")
        | keyword!("and")
        | keyword!("as")
        | keyword!("break")
        | keyword!("continue")
        | keyword!("else")
        | keyword!("for")
        | keyword!("if")
        | keyword!("imm")
        | keyword!("impl")
        | keyword!("in")
        | keyword!("len")
        | keyword!("loop")
        | keyword!("match")
        | keyword!("mod")
        | keyword!("mut")
        | keyword!("newtype")
        | keyword!("not")
        | keyword!("once")
        | keyword!("or")
        | keyword!("pub")
        | keyword!("return")
        | keyword!("share")
        | keyword!("trait")
        | keyword!("undef")
        | keyword!("where")
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
        assert_eq!(keyword("if").easy_parse("if"), Ok(((), "")));
    }
    #[test]
    fn non_keyword() {
        assert!(keyword("if").easy_parse("if_false").is_err());
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
