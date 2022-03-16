use crate::{
    expr::{
        array::{array, range},
        infix::{expr_0, expr_6, infix_expr_op, print_infix_sizes},
        integer::integer_u64,
        record::{print_record_sizes, record},
        string::{char_literal, string_literal},
        tuple::{print_tuple_sizes, tuple},
    },
    ident_keyword::{ident, keyword},
    lex,
    pattern::parameter,
    size_of,
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

combine::parser! {
    fn literal[I]()(I) -> Literal
    where [
        I: Stream<Token = char>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        choice((
            char_literal().map(Literal::UInt),
            float::float().map(Literal::Float),
            integer_u64().map(Literal::UInt),
            attempt(keyword("false")).with(value(Literal::False)),
            attempt(keyword("true") ).with(value(Literal::True)),
        ))
    }
}
fn jump<I, T>() -> impl Parser<I, Output = Jump<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    choice((
        lex(keyword("break"))
            .with(optional(expr(0)))
            .map(|expr| Jump::Break(expr.map(Box::new))),
        lex(keyword("continue")).map(|_| Jump::Continue),
        lex(keyword("return"))
            .with(optional(expr(0)))
            .map(|expr| Jump::Return(expr.map(Box::new))),
    ))
}
fn unary<I, T>() -> impl Parser<I, Output = Unary<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
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
fn tag<I, T>() -> impl Parser<I, Output = Tag<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    lex(char('@'))
        .with((lex(ident()), optional(expr(6))))
        .map(|(tag, expr)| Tag {
            tag,
            expr: expr.map(Box::new),
        })
}
fn fun<I, T>() -> impl Parser<I, Output = Fun<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    (attempt(parameter().skip(lex(string("=>")))), expr(0)).map(|(param, body)| Fun {
        param,
        body: Box::new(body),
    })
}
fn array_range<I, T>() -> impl Parser<I, Output = Expr<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    choice((
        attempt(range()).map(Expr::ArrayRange),
        array().map(Expr::Array),
    ))
}
fn tuple_record_group<I, T>() -> impl Parser<I, Output = Expr<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    choice((
        attempt((lex(char('(')), lex(char(')'))))
            .map(|_| Expr::Unit)
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
fn prefix_expr_<I, T>() -> impl Parser<I, Output = Expr<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
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
    fn prefix_expr[I, T]()(I) -> Expr< T>
    where [
        I: Stream<Token = char>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        T: Default,
    ] {
        prefix_expr_()
    }
}
fn expr_<I, T>(precedence: u8) -> impl Parser<I, Output = Expr<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    match precedence {
        0 => expr_0().left().left(),
        1..=5 => chainl1(expr(precedence + 1), attempt(infix_expr_op(precedence)))
            .right()
            .left(),
        6 => expr_6().left().right(),
        7.. => prefix_expr().right().right(),
    }
}
combine::parser! {
    pub(crate) fn expr[I, T](precedence: u8)(I) -> Expr< T>
    where [
        I: Stream<Token = char>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        T: Default,
    ] {
        expr_(*precedence)
    }
}
pub(crate) fn print_expr_sizes() {
    print_infix_sizes();
    print_record_sizes();
    print_tuple_sizes();
    println!(
        "{}: {}",
        concat!(module_path!(), "::prefix_expr_"),
        size_of(&prefix_expr_::<&str, ()>()),
    );
    println!(
        "{}: {}",
        concat!(module_path!(), "::expr_"),
        size_of(&expr_::<&str, ()>(0)),
    );
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
                place: Box::new(var_place("foo")),
                expr: Box::new(Expr::Assign(
                    vec![Assign {
                        place: Box::new(var_place("bar")),
                        expr: Box::new(var_expr("baz")),
                    }]
                    .into(),
                )),
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
