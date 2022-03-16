use crate::{
    expr::integer::{integer_i64, integer_u64},
    ident_keyword::{ident, keyword},
    lex, sep_optional_between,
};
use combine::{
    attempt, between, choice, error::StreamError, optional, parser::char::char, sep_end_by,
    stream::StreamErrorFor, ParseError, Parser, Stream,
};
use hir::pattern::{ListPattern, ListWithRest, Pattern, RecordPattern, TaggedPattern, Var};
use std::collections::HashMap;
use string_cache::DefaultAtom;

fn var<I, T>() -> impl Parser<I, Output = Var<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    (
        optional(attempt(lex(keyword("ref")))),
        optional(attempt(lex(keyword("mut")))),
        lex(ident()),
    )
        .map(|(bind_to_ref, mutability, ident)| Var {
            ident,
            mutable: mutability.is_some(),
            bind_to_ref: bind_to_ref.is_some(),
            ty: T::default(),
        })
}
fn list<I, T>() -> impl Parser<I, Output = ListPattern<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    sep_optional_between(pattern, lex(char('*')).with(pattern()), || lex(char(','))).map(
        |(left, rest_right)| {
            let left: Vec<_> = left;
            match rest_right {
                Some((rest, right)) => ListPattern::ListWithRest(ListWithRest {
                    left: left.into(),
                    rest: Box::new(rest),
                    right: right.into(),
                }),
                None => ListPattern::List(left.into()),
            }
        },
    )
}
fn array<I, T>() -> impl Parser<I, Output = ListPattern<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    between(lex(char('[')), lex(char(']')), list()).expected("array pattern")
}
fn tuple<I, T>() -> impl Parser<I, Output = ListPattern<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    between(lex(char('(')), lex(char(')')), list()).expected("tuple pattern")
}
pub(crate) fn parameter<I, T>() -> impl Parser<I, Output = Box<[Var<T>]>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    between(
        lex(char('(')),
        lex(char(')')),
        sep_end_by(var(), lex(char(','))),
    )
    .map(Vec::into)
    .expected("parameter")
}
fn record<I, T>() -> impl Parser<I, Output = RecordPattern<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    let field = || {
        (optional(lex(ident())), lex(char('=')).with(pattern())).and_then(|(name, pattern)| {
            match name.map(DefaultAtom::from).or_else(|| pattern.field_name()) {
                Some(name) => Ok((name, pattern)),
                None => Err(<StreamErrorFor<I>>::message_static_message(
                    "couldn't infer field name",
                )),
            }
        })
    };
    // TODO: handle duplicate name as error
    between(
        lex(char('(')),
        lex(char(')')),
        sep_optional_between(field, lex(char('*')).with(pattern()), || lex(char(','))),
    )
    .map(|(left, rest_right)| match rest_right {
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
    .expected("record pattern")
}
fn pattern_<I, T>() -> impl Parser<I, Output = Pattern<T>>
where
    I: Stream<Token = char>,
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
            .with(pattern())
            .map(|pattern| Pattern::Ref(Box::new(pattern))),
        // TODO: this may not be able to parse i64::MIN
        lex(char('-'))
            .with(integer_i64())
            .map(|num| Pattern::Int(-num)),
        integer_u64().map(Pattern::UInt),
        attempt(between(lex(char('(')), lex(char(')')), pattern())).expected("group"),
        record().map(Pattern::Record),
        tuple().map(Pattern::Tuple),
        array().map(Pattern::Array),
        attempt(lex(keyword("_"))).map(|_| Pattern::Ignore),
        attempt(lex(keyword("true"))).map(|_| Pattern::True),
        attempt(lex(keyword("false"))).map(|_| Pattern::False),
        var().map(Pattern::Var),
    ))
}
combine::parser! {
    pub(crate) fn pattern[I, T]()(I) -> Pattern< T>
    where [
        I: Stream<Token = char>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        T: Default,
    ] {
        pattern_()
    }
}
