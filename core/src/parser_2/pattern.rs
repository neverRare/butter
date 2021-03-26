use crate::ast::pattern::Pattern;
use crate::parser_2::ident_keyword::ident;
use crate::parser_2::ident_keyword::keyword;
use crate::parser_2::lex;
use combine::attempt;
use combine::choice;
use combine::parser;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

fn pattern_<'a, I>() -> impl Parser<I, Output = Pattern<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
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
