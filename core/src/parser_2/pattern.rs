use crate::ast::pattern::Pattern;
use crate::parser_2::ident_keyword::ident;
use crate::parser_2::ident_keyword::keyword;
use crate::parser_2::lex;
use combine::attempt;
use combine::between;
use combine::choice;
use combine::parser;
use combine::parser::char::char;
use combine::sep_by;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

fn array<'a, I>() -> impl Parser<I, Output = Vec<Pattern<'a>>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    between(
        lex(char('[')),
        lex(char(']')),
        sep_by(pattern(), lex(char(','))),
    )
}
fn pattern_<'a, I>() -> impl Parser<I, Output = Pattern<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        array().map(Pattern::Array),
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
