use crate::{expr::expr, ident_keyword::ident, lex, sep_optional_between, size_of};
use combine::{
    between, error::StreamError, optional, parser::char::char, stream::StreamErrorFor, ParseError,
    Parser, RangeStream,
};
use hir::expr::{Field, Record, RecordWithSplat};

// TODO: handle duplicate name
pub(crate) fn record<'a, I, T>() -> impl Parser<I, Output = Record<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    let field = || {
        (optional(lex(ident())), lex(char('=')).with(expr(0))).and_then(|(name, expr)| {
            match name.or_else(|| expr.field_name()) {
                Some(name) => Ok(Field { name, expr }),
                None => Err(<StreamErrorFor<I>>::unexpected_static_message(
                    "couldn't infer field name",
                )),
            }
        })
    };
    let fields = || {
        sep_optional_between(field, lex(char('*')).with(expr(0)), || lex(char(','))).map(
            |(left, rest_right)| {
                let left: Vec<_> = left;
                match rest_right {
                    Some((rest, right)) => Record::RecordWithSplat(RecordWithSplat {
                        left: left.into(),
                        splat: Box::new(rest),
                        right: right.into(),
                    }),
                    None => Record::Record(left.into()),
                }
            },
        )
    };
    between(lex(char('(')), lex(char(')')), fields()).expected("record")
}
pub(crate) fn print_record_sizes() {
    println!(
        "{}: {}",
        concat!(module_path!(), "::record"),
        size_of(&record::<&str, ()>()),
    );
}
