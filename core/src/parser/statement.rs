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

fn statement_<'a, I, P>(end_look_ahead: P) -> impl Parser<I, Output = (Statement<'a>, bool)>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    P: Parser<I, Output = ()>,
{
    let declare = || {
        (
            attempt(pattern().skip(lex(char('=')))),
            expr(0).skip(lex(char(';'))),
        )
            .map(|(pattern, expr)| Declare {
                pattern,
                expr: Box::new(expr),
            })
    };
    choice((
        (control_flow(), optional(lex(char(';'))))
            .map(|(expr, semicolon)| (Statement::Expr(expr), semicolon.is_some())),
        declare().map(|declare| (Statement::Declare(declare), true)),
        (
            expr(0).map(Statement::Expr),
            choice((
                lex(char(';')).map(|_| true),
                look_ahead(end_look_ahead).map(|_| false),
            )),
        ),
    ))
}
parser! {
    pub fn statement['a, I, P](end_look_ahead: P)(I) -> (Statement<'a>, bool)
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        P: Parser<I, Output = ()>,
    ] {
        statement_(end_look_ahead)
    }
}
