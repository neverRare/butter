use combine::attempt;
use combine::parser::char::space;
use combine::parser::char::string;
use combine::parser::range::take_while;
use combine::skip_many;
use combine::skip_many1;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

mod expr;
mod ident_keyword;
mod pattern;

fn comments<'a, I>() -> impl Parser<I, Output = ()>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    skip_many1((attempt(string("--")), take_while(|ch: char| ch != '\n')).expected("comment"))
}
fn insignificants<'a, I>() -> impl Parser<I, Output = ()>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    skip_many(skip_many1(space()).or(comments()))
}
fn lex<'a, I, P>(parser: P) -> impl Parser<I, Output = P::Output>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    P: Parser<I>,
{
    parser.skip(insignificants())
}
#[cfg(test)]
mod test {
    use crate::parser::insignificants;
    use combine::Parser;

    #[test]
    fn insignificant() {
        assert_eq!(
            insignificants()
                .parse("  -- comment\n  -- more comment")
                .unwrap(),
            ((), "")
        )
    }
}
