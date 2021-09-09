use crate::{
    expr::{expr, Expr},
    ident_keyword::ident,
    lex,
};
use combine::{
    between, choice, optional, parser::char::char, sep_end_by, ParseError, Parser, RangeStream,
};
use hir::expr::{Field, FieldSplat, PlaceExpr};

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
        field().map(|(name, expr)| FieldSplat::Field(Field { name, expr })),
        splat().map(FieldSplat::Splat),
    ))
}
// TODO: handle duplicate name
pub(crate) fn record<'a, I, T>() -> impl Parser<I, Output = Box<[FieldSplat<'a, T>]>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    let fields = || sep_end_by(field_splat(), lex(char(',')));
    between(lex(char('(')), lex(char(')')), fields())
        .map(Vec::into)
        .expected("record")
}
