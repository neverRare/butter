use crate::{
    expr::{expr, Expr},
    ident_keyword::ident,
    lex,
};
use combine::{
    between, choice, optional, parser::char::char, sep_end_by, ParseError, Parser, RangeStream,
};
use hir::expr::{PlaceExpr, Record};
use std::collections::HashMap;

struct RecordExtend<'a, T> {
    splats: Vec<Expr<'a, T>>,
    fields: HashMap<&'a str, Expr<'a, T>>,
}
impl<'a, T> Default for RecordExtend<'a, T> {
    fn default() -> Self {
        Self {
            splats: Vec::new(),
            fields: HashMap::new(),
        }
    }
}
impl<'a, T> RecordExtend<'a, T> {
    fn into_struct(self) -> Record<'a, T> {
        let mut fields = self.fields;
        fields.shrink_to_fit();
        Record {
            splats: self.splats.into(),
            fields,
        }
    }
}
impl<'a, T> Extend<FieldSplat<'a, T>> for RecordExtend<'a, T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = FieldSplat<'a, T>>,
    {
        let iter = iter.into_iter();
        let (min_count, _) = iter.size_hint();
        self.fields.reserve(min_count);
        for field in iter {
            match field {
                FieldSplat::Field(name, expr) => {
                    self.fields.insert(name, expr);
                }
                FieldSplat::Splat(expr) => self.splats.push(expr),
            }
        }
    }
}
enum FieldSplat<'a, T> {
    Field(&'a str, Expr<'a, T>),
    Splat(Expr<'a, T>),
}
fn field_splat<'a, I, T>() -> impl Parser<I, Output = FieldSplat<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    let field = || {
        (lex(ident()), optional(lex(char('=')).with(expr(0))))
            .map(|(name, expr)| (name, expr.unwrap_or(Expr::Place(PlaceExpr::Var(name)))))
    };
    let splat = || lex(char('*')).with(expr(0));
    choice((
        field().map(|(name, expr)| FieldSplat::Field(name, expr)),
        splat().map(FieldSplat::Splat),
    ))
}
// TODO: handle duplicate name
pub(crate) fn record<'a, I, T>() -> impl Parser<I, Output = Record<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    let fields = || sep_end_by(field_splat(), lex(char(',')));
    between(lex(char('(')), lex(char(')')), fields()).map(RecordExtend::into_struct)
}
