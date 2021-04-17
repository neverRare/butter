use crate::expr::compound::Struct;
use crate::parser::expr::expr;
use crate::parser::expr::Expr;
use crate::parser::ident_keyword::ident;
use crate::parser::lex;
use combine::between;
use combine::choice;
use combine::optional;
use combine::parser::char::char;
use combine::sep_end_by;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

#[derive(Default)]
struct StructExtend<'a>(Struct<'a>);
impl<'a> Extend<FieldSplat<'a>> for StructExtend<'a> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = FieldSplat<'a>>,
    {
        let Self(record) = self;
        let iter = iter.into_iter();
        let (min_count, _) = iter.size_hint();
        record.fields.reserve(min_count);
        for field in iter {
            match field {
                FieldSplat::Field(name, expr) => {
                    record.fields.insert(name, expr);
                }
                FieldSplat::Splat(expr) => record.splats.push(expr),
            }
        }
    }
}
enum FieldSplat<'a> {
    Field(&'a str, Expr<'a>),
    Splat(Expr<'a>),
}
fn field_splat<'a, I>() -> impl Parser<I, Output = FieldSplat<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let field = || {
        (lex(ident()), optional(lex(char('=')).with(expr(0))))
            .map(|(name, expr)| (name, expr.unwrap_or(Expr::Var(name))))
    };
    let splat = || lex(char('*')).with(expr(0));
    choice((
        field().map(|(name, expr)| FieldSplat::Field(name, expr)),
        splat().map(FieldSplat::Splat),
    ))
}
// TODO: handle duplicate name
pub fn record<'a, I>() -> impl Parser<I, Output = Struct<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let fields = || sep_end_by(field_splat(), lex(char(',')));
    between(lex(char('(')), lex(char(')')), fields()).map(|StructExtend(record)| record)
}
