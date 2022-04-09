use crate::{expr::expr, ident_keyword::ident, lex, sep_optional_between};
use combine::{
    between, error::StreamError, optional, parser::char::char, stream::StreamErrorFor, ParseError,
    Parser, Stream,
};
use hir::{
    expr::{Field, Record, RecordWithSplat},
    Atom,
};

pub(crate) fn record<T, I>() -> impl Parser<I, Output = Record<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default + Clone,
{
    let field = || {
        (optional(lex(ident())), lex(char('=')).with(expr(0))).and_then(|(name, expr)| {
            match name.map(Atom::from).or_else(|| expr.field_name()) {
                Some(name) => Ok(Field { name, expr }),
                None => Err(<StreamErrorFor<I>>::message_static_message(
                    "couldn't infer field name",
                )),
            }
        })
    };
    let fields = || {
        sep_optional_between(field, lex(char('*')).with(expr(0)), || lex(char(',')))
            .map(|(left, rest_right)| {
                let left: Vec<_> = left;
                match rest_right {
                    Some((rest, right)) => Record::RecordWithSplat(RecordWithSplat {
                        left: left.into(),
                        splat: Box::new(rest),
                        right: right.into(),
                    }),
                    None => Record::Record(left.into()),
                }
            })
            .and_then(|record| {
                if record.all_name_unique() {
                    Ok(record)
                } else {
                    Err(<StreamErrorFor<I>>::message_static_message(
                        "duplicate field name",
                    ))
                }
            })
    };
    between(lex(char('(')), lex(char(')')), fields()).expected("record")
}
