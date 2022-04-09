use crate::{
    expr::{
        array::{array, range},
        infix::{expr_0, expr_6, infix_expr_op},
        integer::integer_u64,
        record::record,
        string::{char_literal, string_literal},
        tuple::tuple,
    },
    ident_keyword::{ident, keyword},
    lex,
    pattern::parameter,
};
use combine::{
    attempt, between, chainl1, choice, optional,
    parser::char::{char, string},
    value, ParseError, Parser, Stream,
};
use hir::expr::{Element, ElementKind, Expr, Fun, Jump, Literal, PlaceExpr, Tag, Unary, UnaryType};

mod array;
pub(crate) mod control_flow;
mod float;
mod infix;
pub(crate) mod integer;
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
        float::float().map(Literal::Float),
        integer_u64().map(Literal::UInt),
        // TODO: minus integer parser
        attempt(keyword("false")).with(value(Literal::False)),
        attempt(keyword("true")).with(value(Literal::True)),
    ))
}
fn jump<T, I>() -> impl Parser<I, Output = Jump<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default + Clone,
{
    choice((
        lex(keyword("break"))
            .with(optional(expr(0)))
            .map(|expr| Jump::Break(expr.map(Box::new))),
        lex(keyword("continue")).with(value(Jump::Continue)),
        lex(keyword("return"))
            .with(optional(expr(0)))
            .map(|expr| Jump::Return(expr.map(Box::new))),
    ))
}
fn unary<T, I>() -> impl Parser<I, Output = Unary<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default + Clone,
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
fn tag<T, I>() -> impl Parser<I, Output = Tag<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default + Clone,
{
    lex(char('@'))
        .with((lex(ident()), optional(expr(6))))
        .map(|(tag, expr)| Tag {
            tag,
            expr: expr.map(Box::new),
        })
}
fn fun<T, I>() -> impl Parser<I, Output = Fun<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default + Clone,
{
    (attempt(parameter().skip(lex(string("=>")))), expr(0)).map(|(param, body)| Fun {
        param,
        body: Box::new(body),
    })
}
fn array_range<T, I>() -> impl Parser<I, Output = Expr<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default + Clone,
{
    choice((
        attempt(range()).map(Expr::ArrayRange),
        array().map(Expr::Array),
    ))
}
fn tuple_record_group<T, I>() -> impl Parser<I, Output = Expr<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default + Clone,
{
    choice((
        attempt((lex(char('(')), lex(char(')'))))
            .with(value(Expr::Unit))
            .silent(),
        attempt(between(
            (lex(char('(')), lex(char('*'))),
            (optional(lex(char(','))), lex(char(')'))),
            expr(0),
        ))
        .map(|expr| Expr::Splat(Box::new(expr)))
        .silent(),
        attempt(between(lex(char('(')), lex(char(')')), expr(0))).expected("group"),
        attempt(tuple()).map(Expr::Tuple),
        record().map(Expr::Record),
    ))
}
fn prefix_expr_<T, I>() -> impl Parser<I, Output = Expr<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default + Clone,
{
    choice((
        fun().map(Expr::Fun),
        tuple_record_group(),
        array_range(),
        lex(string_literal()).map(|vec| {
            let vec = vec
                .into_iter()
                .map(|byte| Element {
                    expr: Expr::Literal(Literal::UInt(byte as u64)),
                    kind: ElementKind::Element,
                })
                .collect();
            Expr::Array(vec)
        }),
        unary().map(Expr::Unary),
        tag().map(Expr::Tag),
        attempt(lex(ident())).map(|ident| Expr::Place(PlaceExpr::Var(ident))),
        control_flow::control_flow().map(Expr::ControlFlow),
        lex(literal()).map(Expr::Literal),
        jump().map(Expr::Jump),
    ))
}
combine::parser! {
    fn prefix_expr[T, I]()(I) -> Expr<T>
    where [
        I: Stream<Token = char>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        T: Default + Clone,
    ] {
        prefix_expr_()
    }
}
fn expr_<T, I>(precedence: u8) -> impl Parser<I, Output = Expr<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default + Clone,
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
    pub(crate) fn expr[T, I](precedence: u8)(I) -> Expr<T>
    where [
        I: Stream<Token = char>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        T: Default + Clone,
    ] {
        expr_(*precedence)
    }
}
#[cfg(test)]
mod test {
    use crate::{
        expr::{expr, Expr},
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
        let expected = Expr::Binary(Binary {
            kind: BinaryType::Add,
            left: Box::new(var_expr("foo")),
            right: Box::new(Expr::Binary(Binary {
                kind: BinaryType::Multiply,
                left: Box::new(var_expr("bar")),
                right: Box::new(var_expr("baz")),
            })),
        });
        assert_eq!(expr(0).easy_parse(src), Ok((expected, "")));
        let src = "foo * bar + baz";
        let expected = Expr::Binary(Binary {
            kind: BinaryType::Add,
            left: Box::new(Expr::Binary(Binary {
                kind: BinaryType::Multiply,
                left: Box::new(var_expr("foo")),
                right: Box::new(var_expr("bar")),
            })),
            right: Box::new(var_expr("baz")),
        });
        assert_eq!(expr(0).easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn right_associative() {
        let src = "foo <- bar <- baz";
        let expected = Expr::Assign(
            vec![Assign {
                place: var_place("foo"),
                expr: Expr::Assign(
                    vec![Assign {
                        place: var_place("bar"),
                        expr: var_expr("baz"),
                    }]
                    .into(),
                ),
            }]
            .into(),
        );
        assert_eq!(expr(0).easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn ignore_higher_precedence() {
        let src = "foo + bar";
        let expected: Expr<()> = var_expr("foo");
        let left = "+ bar";
        assert_eq!(expr(6).easy_parse(src), Ok((expected, left)));
    }
    #[test]
    fn ignore_range() {
        let src = "foo..";
        let expected: Expr<()> = var_expr("foo");
        let left = "..";
        assert_eq!(expr(0).easy_parse(src), Ok((expected, left)));
    }
}
