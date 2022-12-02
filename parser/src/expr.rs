use crate::{
    control_flow::control_flow,
    expr::{
        array::{array, range},
        infix::{expr_0, expr_6, infix_expr_op},
        record::record,
        string::{char_literal, string_literal},
        tuple::tuple,
    },
    ident_keyword::{ident, keyword},
    lex,
    number::{float, integer_u64},
    pattern::parameter,
};
use combine::{
    attempt, between, chainl1, choice, optional,
    parser::char::{char, string},
    value, ParseError, Parser, Stream,
};
use hir::expr::{
    Element, ElementKind, Expr, ExprKind, Fun, Jump, Literal, PlaceExpr, Tag, Unary, UnaryType,
};

mod array;
mod infix;
mod record;
mod string;
mod tuple;

fn literal<I>() -> impl Parser<I, Output = Literal>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        char_literal().map(Literal::UInt),
        float().map(Literal::Float),
        integer_u64().map(Literal::UInt),
        // TODO: minus integer parser
        attempt(keyword("false")).with(value(Literal::False)),
        attempt(keyword("true")).with(value(Literal::True)),
    ))
}
fn jump<I>() -> impl Parser<I, Output = Jump<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        lex(keyword("break"))
            .with(optional(expr(0)))
            .map(|expr| expr.map(Box::new))
            .map(Jump::Break),
        lex(keyword("continue")).with(value(Jump::Continue)),
        lex(keyword("return"))
            .with(optional(expr(0)))
            .map(|expr| expr.map(Box::new))
            .map(Jump::Return),
    ))
}
fn unary<I>() -> impl Parser<I, Output = Unary<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let kind = || {
        choice((
            char('!').with(value(UnaryType::Not)),
            char('&').with(value(UnaryType::Ref)),
            char('-').with(value(UnaryType::Minus)),
            char('>').with(value(UnaryType::Move)),
            attempt(keyword("clone")).with(value(UnaryType::Clone)),
        ))
    };
    (lex(kind()), expr(6)).map(|(kind, expr)| Unary {
        kind,
        expr: Box::new(expr),
    })
}
fn tag<I>() -> impl Parser<I, Output = Tag<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    lex(char('@'))
        .with((lex(ident()), optional(expr(6))))
        .map(|(tag, expr)| Tag {
            tag,
            expr: expr.map(Box::new),
        })
}
fn fun<I>() -> impl Parser<I, Output = Fun<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (attempt(parameter().skip(lex(string("=>")))), expr(0)).map(|(param, body)| Fun {
        param,
        body: Box::new(body),
    })
}
fn array_range<I>() -> impl Parser<I, Output = Expr<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        attempt(range()).map(ExprKind::ArrayRange),
        array().map(ExprKind::Array),
    ))
    .map(ExprKind::into_untyped)
}
fn tuple_record_group<I>() -> impl Parser<I, Output = Expr<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        attempt((lex(char('(')), lex(char(')'))))
            .with(value(ExprKind::Unit.into_untyped()))
            .silent(),
        attempt(between(
            (lex(char('(')), lex(char('*'))),
            (optional(lex(char(','))), lex(char(')'))),
            expr(0),
        ))
        .map(Box::new)
        .map(ExprKind::Splat)
        .map(ExprKind::into_untyped)
        .silent(),
        attempt(between(lex(char('(')), lex(char(')')), expr(0))).expected("group"),
        attempt(tuple())
            .map(ExprKind::Tuple)
            .map(ExprKind::into_untyped),
        record().map(ExprKind::Record).map(ExprKind::into_untyped),
    ))
}
fn prefix_expr_<I>() -> impl Parser<I, Output = Expr<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        fun().map(|fun| ExprKind::Fun(fun).into_untyped()),
        tuple_record_group(),
        array_range(),
        lex(string_literal()).map(|vec| {
            let vec = vec
                .into_iter()
                .map(|byte| Element {
                    expr: ExprKind::Literal(Literal::UInt(byte as u64)).into_untyped(),
                    kind: ElementKind::Element,
                })
                .collect();
            ExprKind::Array(vec).into_untyped()
        }),
        unary().map(ExprKind::Unary).map(ExprKind::into_untyped),
        tag().map(ExprKind::Tag).map(ExprKind::into_untyped),
        attempt(lex(ident()))
            .map(PlaceExpr::Var)
            .map(ExprKind::Place)
            .map(ExprKind::into_untyped),
        control_flow()
            .map(ExprKind::ControlFlow)
            .map(ExprKind::into_untyped),
        lex(literal())
            .map(ExprKind::Literal)
            .map(ExprKind::into_untyped),
        jump().map(ExprKind::Jump).map(ExprKind::into_untyped),
    ))
}
combine::parser! {
    fn prefix_expr[I]()(I) -> Expr<()>
    where [
        I: Stream<Token = char>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        prefix_expr_()
    }
}
fn expr_<I>(precedence: u8) -> impl Parser<I, Output = Expr<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    match precedence {
        0 => expr_0().left().left(),
        1..=5 => chainl1(expr(precedence + 1), lex(infix_expr_op(precedence)))
            .right()
            .left(),
        6 => expr_6().left().right(),
        7.. => prefix_expr().right().right(),
    }
}
combine::parser! {
    pub(super) fn expr[I](precedence: u8)(I) -> Expr<()>
    where [
        I: Stream<Token = char>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        expr_(*precedence)
    }
}
#[cfg(test)]
mod test {
    use crate::{
        expr::{expr, ExprKind},
        test::{var_expr, var_place},
    };
    use combine::EasyParser;
    use hir::expr::{Assign, Binary, BinaryType};

    #[test]
    fn group() {
        let src = "(foo)";
        let expected = var_expr("foo");
        assert_eq!(expr(0).easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn precedence() {
        let src = "foo + bar * baz";
        let expected = ExprKind::Binary(Binary {
            kind: BinaryType::Add,
            left: Box::new(var_expr("foo")),
            right: Box::new(
                ExprKind::Binary(Binary {
                    kind: BinaryType::Multiply,
                    left: Box::new(var_expr("bar")),
                    right: Box::new(var_expr("baz")),
                })
                .into_untyped(),
            ),
        })
        .into_untyped();
        assert_eq!(expr(0).easy_parse(src), Ok((expected, "")));
        let src = "foo * bar + baz";
        let expected = ExprKind::Binary(Binary {
            kind: BinaryType::Add,
            left: Box::new(
                ExprKind::Binary(Binary {
                    kind: BinaryType::Multiply,
                    left: Box::new(var_expr("foo")),
                    right: Box::new(var_expr("bar")),
                })
                .into_untyped(),
            ),
            right: Box::new(var_expr("baz")),
        })
        .into_untyped();
        assert_eq!(expr(0).easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn right_associative() {
        let src = "foo <- bar <- baz";
        let expected = ExprKind::Assign(
            vec![Assign {
                place: var_place("foo"),
                expr: ExprKind::Assign(
                    vec![Assign {
                        place: var_place("bar"),
                        expr: var_expr("baz"),
                    }]
                    .into(),
                )
                .into_untyped(),
            }]
            .into(),
        )
        .into_untyped();
        assert_eq!(expr(0).easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn ignore_higher_precedence() {
        let src = "foo + bar";
        let expected = var_expr("foo");
        let left = "+ bar";
        assert_eq!(expr(6).easy_parse(src), Ok((expected, left)));
    }
    #[test]
    fn ignore_range() {
        let src = "foo..";
        let expected = var_expr("foo");
        let left = "..";
        assert_eq!(expr(0).easy_parse(src), Ok((expected, left)));
    }
}
