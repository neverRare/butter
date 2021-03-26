use crate::ast::pattern::ArrayWithRest;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::PatternField;
use crate::ast::pattern::StructPattern;
use crate::parser_2::ident_keyword::ident;
use crate::parser_2::ident_keyword::keyword;
use crate::parser_2::lex;
use combine::attempt;
use combine::choice;
use combine::optional;
use combine::parser;
use combine::parser::char::char;
use combine::sep_by;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

fn elements<'a, I>() -> impl Parser<I, Output = Vec<Pattern<'a>>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    sep_by(pattern(), lex(char(',')))
}
fn rest<'a, I>() -> impl Parser<I, Output = Pattern<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (lex(char('*')), pattern()).map(|(_, pattern)| pattern)
}
fn array<'a, I>() -> impl Parser<I, Output = Pattern<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        lex(char('[')),
        elements(),
        optional((rest(), optional((lex(char(',')), elements())))),
        lex(char(']')),
    )
        .map(|(_, left, rest_right, _)| match rest_right {
            Some((rest, right)) => {
                let right = match right {
                    Some((_, right)) => right,
                    None => vec![],
                };
                Pattern::ArrayWithRest(ArrayWithRest {
                    left,
                    rest: Box::new(rest),
                    right,
                })
            }
            None => Pattern::Array(left),
        })
}
fn field<'a, I>() -> impl Parser<I, Output = PatternField<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (lex(ident()), optional((lex(char('=')), pattern()))).map(|(name, pattern)| {
        let pattern = match pattern {
            Some((_, pattern)) => pattern,
            None => Pattern::Var(name),
        };
        PatternField {
            name,
            content: pattern,
        }
    })
}
fn fields<'a, I>() -> impl Parser<I, Output = Vec<PatternField<'a>>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    sep_by(field(), lex(char(',')))
}
fn record<'a, I>() -> impl Parser<I, Output = StructPattern<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        lex(char('(')),
        fields(),
        optional((rest(), optional((lex(char(',')), fields())))),
        lex(char(')')),
    )
        .map(|(_, left, rest_right, _)| match rest_right {
            Some((rest, right)) => {
                let fields = match right {
                    Some((_, mut right)) => {
                        let mut fields = left;
                        fields.append(&mut right);
                        fields
                    }
                    None => left,
                };
                StructPattern {
                    fields,
                    rest: Some(Box::new(rest)),
                }
            }
            None => StructPattern {
                fields: left,
                rest: None,
            },
        })
}
fn pattern_<'a, I>() -> impl Parser<I, Output = Pattern<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        array(),
        record().map(Pattern::Struct),
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
