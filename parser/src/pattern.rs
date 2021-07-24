use crate::expr::integer::integer_i64;
use crate::expr::integer::integer_u64;
use crate::ident_keyword::ident;
use crate::ident_keyword::keyword;
use crate::lex;
use combine::attempt;
use combine::between;
use combine::choice;
use combine::many;
use combine::optional;
use combine::parser;
use combine::parser::char::char;
use combine::sep_end_by;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;
use hir::pattern::ArrayWithRest;
use hir::pattern::Pattern;
use hir::pattern::RecordPattern;
use hir::pattern::TaggedPattern;
use hir::pattern::Var;
use std::collections::HashMap;

fn optional_rest<'a, I, EP, RP, C>(
    left: char,
    right: char,
    element: fn() -> EP,
    rest: fn() -> RP,
) -> impl Parser<I, Output = (C, Option<(RP::Output, C)>)>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    EP: Parser<I>,
    RP: Parser<I>,
    C: Extend<EP::Output> + Default,
{
    let no_rest = move || sep_end_by(element(), lex(char(',')));
    let have_rest = move || {
        (
            many(element().skip(lex(char(',')))),
            (lex(char('*'))).with(rest()),
            optional(lex(char(',')).with(no_rest()))
                .map(|right| right.unwrap_or_else(Default::default)),
        )
    };
    let middle = move || {
        choice((
            attempt(have_rest()).map(|(left, rest, right)| (left, Some((rest, right)))),
            no_rest().map(|collection| (collection, None)),
        ))
    };
    between(lex(char(left)), lex(char(right)), middle())
}
fn array<'a, I, T>() -> impl Parser<I, Output = Pattern<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    optional_rest('[', ']', pattern, pattern).map(|(left, rest_right)| {
        let left: Vec<_> = left;
        match rest_right {
            Some((rest, right)) => Pattern::ArrayWithRest(ArrayWithRest {
                left: left.into(),
                rest: Box::new(rest),
                right: right.into(),
            }),
            None => Pattern::Array(left.into()),
        }
    })
}
fn field<'a, I, T>() -> impl Parser<I, Output = (&'a str, Pattern<'a, T>)>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    (lex(ident()), optional(lex(char('=')).with(pattern()))).map(|(name, pattern)| {
        (
            name,
            pattern.unwrap_or_else(|| {
                Pattern::Var(Var {
                    ident: name,
                    mutable: false,
                    bind_to_ref: false,
                    ty: T::default(),
                })
            }),
        )
    })
}
pub fn parameter<'a, I, T>() -> impl Parser<I, Output = HashMap<&'a str, Pattern<'a, T>>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    between(
        lex(char('(')),
        lex(char(')')),
        sep_end_by(field(), lex(char(','))),
    )
}
fn record<'a, I, T>() -> impl Parser<I, Output = RecordPattern<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    // TODO: handle duplicate name as error
    optional_rest('(', ')', field, pattern).map(|(left, rest_right)| match rest_right {
        Some((rest, right)) => {
            let mut fields: HashMap<_, _> = left;
            fields.extend(right);
            RecordPattern {
                fields,
                rest: Some(Box::new(rest)),
            }
        }
        None => RecordPattern {
            fields: left,
            rest: None,
        },
    })
}
fn pattern_<'a, I, T>() -> impl Parser<I, Output = Pattern<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    choice((
        lex(char('@'))
            .with((lex(ident()), optional(pattern())))
            .map(|(tag, pattern)| {
                Pattern::Tag(TaggedPattern {
                    tag,
                    pattern: pattern.map(Box::new),
                })
            }),
        lex(char('&'))
            .with((optional(lex(keyword("mut"))), pattern()))
            .map(|(mutability, pattern)| match mutability {
                Some(_) => Pattern::RefMut(Box::new(pattern)),
                None => Pattern::Ref(Box::new(pattern)),
            }),
        // TODO: this may not be able to parse i64::MIN
        lex(char('-'))
            .with(integer_i64())
            .map(|num| Pattern::Int(-num)),
        integer_u64().map(Pattern::UInt),
        array(),
        attempt(between(lex(char('(')), lex(char(')')), pattern())),
        record().map(Pattern::Record),
        attempt(lex(keyword("_"))).map(|_| Pattern::Ignore),
        attempt(lex(keyword("true"))).map(|_| Pattern::True),
        attempt(lex(keyword("false"))).map(|_| Pattern::False),
        (
            optional(attempt(lex(keyword("ref")))),
            optional(attempt(lex(keyword("mut")))),
            lex(ident()),
        )
            .map(|(bind_to_ref, mutability, ident)| {
                Pattern::Var(Var {
                    ident,
                    mutable: mutability.is_some(),
                    bind_to_ref: bind_to_ref.is_some(),
                    ty: T::default(),
                })
            }),
    ))
}
parser! {
    pub fn pattern['a, I, T]()(I) -> Pattern<'a, T>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        T: Default,
    ] {
        pattern_()
    }
}
