use crate::{
    control_flow::control_flow,
    expr::expr,
    ident_keyword::ident,
    lex,
    pattern::{parameter, pattern},
};
use combine::{
    attempt, choice,
    error::StreamError,
    look_ahead, optional,
    parser::char::{char, string},
    sep_by1,
    stream::StreamErrorFor,
    value, ParseError, Parser, Stream,
};
use hir::{
    expr::{Assign, Expr, ExprKind, Fun},
    statement::{Declare, FunDeclare, Statement},
};

pub(super) enum StatementReturn {
    Statement(Statement<()>),
    Return(Expr<()>),
}
pub(super) fn statement_return<I, P>(end_look_ahead: P) -> impl Parser<I, Output = StatementReturn>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    P: Parser<I>,
{
    let control_flow_statement = || {
        (control_flow(), optional(lex(char(';')))).map(|(control_flow, semicolon)| {
            let expr = ExprKind::ControlFlow(control_flow).into_untyped();
            match semicolon {
                Some(_) => StatementReturn::Statement(Statement::Expr(expr)),
                None => StatementReturn::Return(expr),
            }
        })
    };
    let fun_body = || {
        choice((
            control_flow()
                .skip(optional(lex(char(';'))))
                .map(ExprKind::ControlFlow)
                .map(ExprKind::into_untyped),
            expr(0).skip(lex(char(';'))),
        ))
    };
    let fun_declare = || {
        (
            attempt((ident(), parameter().skip(lex(string("=>"))))),
            fun_body(),
        )
            .map(|((ident, param), body)| {
                Statement::FunDeclare(FunDeclare {
                    ident,
                    fun: Fun {
                        param,
                        body: Box::new(body),
                    },
                    ty: (),
                })
            })
    };
    let place = || {
        expr(1).and_then(|expr| {
            if let ExprKind::Place(place) = expr.expr {
                Ok(place)
            } else {
                Err(<StreamErrorFor<I>>::expected_static_message(
                    "place expression",
                ))
            }
        })
    };
    let declare = || {
        (attempt(pattern().skip(lex(char('=')))), expr(0))
            .skip(lex(char(';')))
            .map(|(pattern, expr)| {
                StatementReturn::Statement(Statement::Declare(Declare { pattern, expr }))
            })
    };
    let parallel_assign = || {
        (
            attempt(sep_by1(place(), lex(char(','))).skip(lex(string("<-")))),
            sep_by1(expr(0), lex(char(','))),
        )
            .and_then(|(place, expr)| {
                let place: Vec<_> = place;
                let expr: Vec<_> = expr;
                if place.len() != expr.len() {
                    return Err(<StreamErrorFor<I>>::message_static_message(
                        "mismatching count of place and value expressions",
                    ));
                }
                let assign = place
                    .into_iter()
                    .zip(expr.into_iter())
                    .map(|(place, expr)| Assign { place, expr })
                    .collect();
                Ok(ExprKind::Assign(assign).into_untyped())
            })
    };
    let expr = || {
        (
            choice((parallel_assign(), expr(0))),
            choice((
                lex(char(';')).with(value(false)),
                look_ahead(end_look_ahead).with(value(true)),
            )),
        )
            .map(|(expr, implicit_return)| {
                if implicit_return {
                    StatementReturn::Return(expr)
                } else {
                    StatementReturn::Statement(Statement::Expr(expr))
                }
            })
    };
    choice((
        control_flow_statement(),
        declare(),
        fun_declare().map(StatementReturn::Statement),
        expr(),
    ))
}
pub(super) fn statement<I>() -> impl Parser<I, Output = Statement<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    statement_return(char(';')).map(|statement_return| match statement_return {
        StatementReturn::Statement(statement) => statement,
        StatementReturn::Return(expr) => Statement::Expr(expr),
    })
}
#[cfg(test)]
mod test {
    use crate::{
        statement::{statement, Assign, ExprKind},
        test::{var_expr, var_place},
        Statement,
    };
    use combine::EasyParser;
    use hir::{
        expr::Literal,
        pattern::{PatternKind, Var},
        statement::Declare,
        Atom,
    };

    #[test]
    fn parallel_assign() {
        let src = "foo, bar <- bar, foo;";
        let expected = Statement::Expr(
            ExprKind::Assign(
                vec![
                    Assign {
                        place: var_place("foo"),
                        expr: var_expr("bar"),
                    },
                    Assign {
                        place: var_place("bar"),
                        expr: var_expr("foo"),
                    },
                ]
                .into(),
            )
            .into_untyped(),
        );
        assert_eq!(statement().easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn chain_assign() {
        let src = "foo <- bar <- baz;";
        let expected: Statement<()> = Statement::Expr(
            ExprKind::Assign(
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
            .into_untyped(),
        );
        assert_eq!(statement().easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn var() {
        let src = "foo = 10;";
        let expected = Statement::Declare(Declare {
            pattern: PatternKind::Var(Var {
                ident: Atom::from("foo"),
                mutable: false,
                bind_to_ref: false,
            })
            .into_untyped(),
            expr: ExprKind::Literal(Literal::UInt(10)).into_untyped(),
        });
        assert_eq!(statement().easy_parse(src), Ok((expected, "")));
    }
}
