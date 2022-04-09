use crate::{
    expr::{control_flow::control_flow, expr},
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
    expr::{Assign, Expr, Fun},
    statement::{Declare, FunDeclare, Statement},
};

pub(crate) enum StatementReturn<T> {
    Statement(Statement<T>),
    Return(Expr<T>),
}
pub(crate) fn statement_return<I, P, T>(
    end_look_ahead: P,
) -> impl Parser<I, Output = StatementReturn<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    P: Parser<I>,
    T: Default + Clone,
{
    let control_flow_statement = || {
        (control_flow(), optional(lex(char(';')))).map(|(control_flow, semicolon)| {
            let expr = Expr::ControlFlow(control_flow);
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
                .map(|control_flow| Expr::ControlFlow(control_flow)),
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
                    ty: T::default(),
                })
            })
    };
    let place = || {
        expr(1).and_then(|expr| {
            if let Expr::Place(place) = expr {
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
                Ok(Expr::Assign(assign))
            })
    };
    let expr = || {
        (
            choice((parallel_assign(), expr(0))),
            choice((
                lex(char(';')).with(value(true)),
                look_ahead(end_look_ahead).with(value(false)),
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
pub(crate) fn statement<T, I>() -> impl Parser<I, Output = Statement<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default + Clone,
{
    statement_return(char(';')).map(|statement_return| match statement_return {
        StatementReturn::Statement(statement) => statement,
        StatementReturn::Return(expr) => Statement::Expr(expr),
    })
}
#[cfg(test)]
mod test {
    use crate::{
        statement::{statement, Assign, Expr},
        test::{var_expr, var_place},
        Statement,
    };
    use combine::EasyParser;
    use hir::{
        expr::Literal,
        pattern::{Pattern, Var},
        statement::Declare,
        Atom,
    };

    #[test]
    fn parallel_assign() {
        let src = "foo, bar <- bar, foo;";
        let expected: Statement<()> = Statement::Expr(Expr::Assign(
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
        ));
        assert_eq!(statement().easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn chain_assign() {
        let src = "foo <- bar <- baz;";
        let expected: Statement<()> = Statement::Expr(Expr::Assign(
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
        ));
        assert_eq!(statement().easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn var() {
        let src = "foo = 10;";
        let expected: Statement<()> = Statement::Declare(Declare {
            pattern: Pattern::Var(Var {
                ident: Atom::from("foo"),
                mutable: false,
                bind_to_ref: false,
                ty: (),
            }),
            expr: Expr::Literal(Literal::UInt(10)),
        });
        assert_eq!(statement().easy_parse(src), Ok((expected, "")));
    }
}
