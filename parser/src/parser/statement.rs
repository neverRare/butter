use crate::expr::control_flow::Fun;
use crate::expr::operator::Assign;
use crate::expr::Expr;
use crate::expr::PlaceExpr;
use crate::parser::expr::control_flow::block;
use crate::parser::expr::control_flow::control_flow;
use crate::parser::expr::expr;
use crate::parser::ident_keyword::ident;
use crate::parser::lex;
use crate::parser::pattern::parameter;
use crate::parser::pattern::pattern;
use crate::statement::Declare;
use crate::statement::FunDeclare;
use crate::statement::Statement;
use combine::attempt;
use combine::choice;
use combine::error::StreamError;
use combine::look_ahead;
use combine::optional;
use combine::parser;
use combine::parser::char::char;
use combine::parser::char::string;
use combine::sep_by1;
use combine::stream::StreamErrorFor;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

pub enum StatementReturn<'a> {
    Statement(Statement<'a>),
    Return(Expr<'a>),
}
fn statement_return_<'a, I, P>(end_look_ahead: P) -> impl Parser<I, Output = StatementReturn<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    P: Parser<I>,
{
    let control_flow = || {
        (control_flow(), optional(lex(char(';')))).map(|(expr, semicolon)| match semicolon {
            Some(_) => StatementReturn::Statement(Statement::Expr(expr)),
            None => StatementReturn::Return(expr),
        })
    };
    let fun_body = || {
        choice((
            block().skip(optional(lex(char(';')))).map(Expr::Block),
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
                })
            })
    };
    let place = || {
        expr(1).and_then(|expr| match PlaceExpr::from_expr(expr) {
            Some(place) => Ok(place),
            None => Err(<StreamErrorFor<I>>::message_static_message(
                "non place expression",
            )),
        })
    };
    let parallel_assign = || {
        (
            attempt(sep_by1(place(), lex(char(','))).skip(lex(string("<-")))),
            sep_by1(expr(1), lex(char(','))).skip(lex(char(';'))),
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
                    .map(|(place, expr)| Assign {
                        place: Box::new(place),
                        expr: Box::new(expr),
                    })
                    .collect();
                Ok(StatementReturn::Statement(Statement::Expr(
                    Expr::ParallelAssign(assign),
                )))
            })
    };
    let declare = || {
        (attempt(pattern().skip(lex(char('=')))), expr(0))
            .skip(lex(char(';')))
            .map(|(pattern, expr)| {
                StatementReturn::Statement(Statement::Declare(Declare {
                    pattern,
                    expr: Box::new(expr),
                }))
            })
    };
    let expr = || {
        (
            expr(0),
            choice((
                lex(char(';')).map(|_| true),
                look_ahead(end_look_ahead).map(|_| false),
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
        control_flow(),
        declare(),
        fun_declare().map(StatementReturn::Statement),
        parallel_assign(),
        expr(),
    ))
}
parser! {
    pub fn statement_return['a, I, P](end_look_ahead: P)(I) -> StatementReturn<'a>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        P: Parser<I>,
    ] {
        statement_return_(end_look_ahead)
    }
}
pub fn statement<'a, I>() -> impl Parser<I, Output = Statement<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    statement_return(char(';')).map(|statement_return| match statement_return {
        StatementReturn::Statement(statement) => statement,
        StatementReturn::Return(expr) => Statement::Expr(expr),
    })
}
