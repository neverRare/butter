use crate::ast::expr::Expr;
use crate::ast::statement::Declare;
use crate::ast::statement::Statement;
use crate::parser::expr::control_flow::control_flow;
use crate::parser::expr::expr;
use crate::parser::lex;
use crate::parser::pattern::pattern;
use combine::attempt;
use combine::choice;
use combine::look_ahead;
use combine::optional;
use combine::parser;
use combine::parser::char::char;
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
    choice((control_flow(), declare(), expr()))
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
pub fn statement<'a, I, P>() -> impl Parser<I, Output = Statement<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    P: Parser<I>,
{
    statement_return(char(';')).map(|statement_return| match statement_return {
        StatementReturn::Statement(statement) => statement,
        StatementReturn::Return(expr) => Statement::Expr(expr),
    })
}
