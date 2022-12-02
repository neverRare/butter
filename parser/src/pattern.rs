use crate::{
    ident_keyword::{ident, keyword},
    lex,
    number::integer_u64,
    sep_optional_between,
};
use combine::{
    attempt, between, choice, error::StreamError, optional, parser::char::char, sep_end_by,
    stream::StreamErrorFor, value, ParseError, Parser, Stream,
};
use hir::{
    pattern::{ListPattern, ListWithRest, Pattern, PatternKind, RecordPattern, TaggedPattern, Var},
    Atom,
};
use std::collections::HashMap;

fn var<I>() -> impl Parser<I, Output = Var>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
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
        })
}
fn list<I>() -> impl Parser<I, Output = ListPattern<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
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
fn array<I>() -> impl Parser<I, Output = ListPattern<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    between(lex(char('[')), lex(char(']')), list()).expected("array pattern")
}
fn tuple<I>() -> impl Parser<I, Output = ListPattern<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    between(lex(char('(')), lex(char(')')), list()).expected("tuple pattern")
}
pub(super) fn parameter<I>() -> impl Parser<I, Output = Pattern<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    between(
        lex(char('(')),
        lex(char(')')),
        sep_end_by(var().map(Var::into_untyped), lex(char(','))),
    )
    .map(Vec::into)
    .map(PatternKind::Param)
    .map(PatternKind::into_untyped)
    .expected("parameter")
}
fn record<I>() -> impl Parser<I, Output = RecordPattern<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let field = || {
        (optional(lex(ident())), lex(char('=')).with(pattern())).and_then(|(name, pattern)| {
            match name.map(Atom::from).or_else(|| pattern.field_name()) {
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
fn pattern_<I>() -> impl Parser<I, Output = Pattern<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        lex(char('@'))
            .with((lex(ident()), pattern()))
            .map(|(tag, pattern)| {
                PatternKind::Tag(TaggedPattern {
                    tag,
                    pattern: Box::new(pattern),
                })
            })
            .map(PatternKind::into_untyped),
        lex(char('&'))
            .with(pattern())
            .map(Box::new)
            .map(PatternKind::Ref)
            .map(PatternKind::into_untyped),
        // TODO: minus integer
        integer_u64()
            .map(PatternKind::UInt)
            .map(PatternKind::into_untyped),
        attempt(between(lex(char('(')), lex(char(')')), pattern())).expected("group"),
        record()
            .map(PatternKind::Record)
            .map(PatternKind::into_untyped),
        tuple()
            .map(PatternKind::Tuple)
            .map(PatternKind::into_untyped),
        array()
            .map(PatternKind::Array)
            .map(PatternKind::into_untyped),
        attempt(lex(keyword("_"))).with(value(PatternKind::Ignore.into_untyped())),
        attempt(lex(keyword("true"))).with(value(PatternKind::True.into_untyped())),
        attempt(lex(keyword("false"))).with(value(PatternKind::False.into_untyped())),
        var().map(PatternKind::Var).map(PatternKind::into_untyped),
    ))
}
combine::parser! {
    pub(super) fn pattern[I]()(I) -> Pattern<()>
    where [
        I: Stream<Token = char>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        pattern_()
    }
}
