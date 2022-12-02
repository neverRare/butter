use crate::{
    expr::expr,
    ident_keyword::keyword,
    lex,
    pattern::pattern,
    statement::{statement_return, StatementReturn},
};
use combine::{
    attempt, between, choice, look_ahead, many, optional,
    parser::char::{char, string},
    value, ParseError, Parser, Stream,
};
use hir::{
    expr::{Block, ControlFlow, Expr, ExprKind, For, If, Match, MatchArm, While},
    statement::Statement,
};

#[derive(Debug, Default, Clone, PartialEq)]
struct StatementExpr {
    statement: Vec<Statement<()>>,
    expr: Option<Expr<()>>,
}
impl Extend<StatementReturn> for StatementExpr {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = StatementReturn>,
    {
        let iter = iter.into_iter();
        let (min_count, _) = iter.size_hint();
        self.statement.reserve(min_count);
        for statement_return in iter {
            self.statement
                .extend(self.expr.take().into_iter().map(Statement::Expr));
            match statement_return {
                StatementReturn::Statement(statement) => self.statement.push(statement),
                StatementReturn::Return(expr) => self.expr = Some(expr),
            }
        }
    }
}
pub(super) fn block<I>() -> impl Parser<I, Output = Block<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    between(
        lex(char('{')),
        lex(char('}')),
        many(statement_return(char('}'))),
    )
    .map(|statement_expr| {
        let StatementExpr { statement, expr } = statement_expr;
        Block {
            statement: statement.into(),
            expr: expr.map(Box::new),
        }
    })
    .expected("block")
}
fn if_<I>() -> impl Parser<I, Output = If<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let else_part = || {
        lex(keyword("else")).with(choice((
            block().map(ControlFlow::Block),
            if_expression().map(ControlFlow::If),
        )))
    };
    attempt(lex(keyword("if")))
        .with((expr(0), block(), optional(else_part())))
        .map(|(condition, body, else_part)| If {
            condition: Box::new(condition),
            body,
            else_part: else_part.map(Box::new),
        })
}
combine::parser! {
    fn if_expression[I]()(I) -> If<()>
    where [
        I: Stream<Token = char>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        if_()
    }
}
fn for_expression<I>() -> impl Parser<I, Output = For<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    attempt(lex(keyword("for")))
        .with((pattern(), lex(keyword("in")), expr(0), block()))
        .map(|(pattern, _, expr, body)| For {
            pattern,
            expr: Box::new(expr),
            body,
        })
}
fn while_expression<I>() -> impl Parser<I, Output = While<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    attempt(lex(keyword("while")))
        .with((expr(0), block()))
        .map(|(condition, body)| While {
            condition: Box::new(condition),
            body,
        })
}
fn loop_expression<I>() -> impl Parser<I, Output = Block<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    attempt(lex(keyword("loop"))).with(block())
}
fn match_expression<I>() -> impl Parser<I, Output = Match<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let arm_expr = || {
        choice((
            attempt(control_flow())
                .skip(optional(lex(char(','))))
                .map(ExprKind::ControlFlow)
                .map(ExprKind::into_untyped),
            expr(0).skip(choice((
                lex(char(',')).with(value(())),
                look_ahead(char('}')).with(value(())),
            ))),
        ))
    };
    let arm = || {
        (pattern().skip(lex(string("=>"))), arm_expr())
            .map(|(pattern, expr)| MatchArm { pattern, expr })
    };
    let body = || {
        between(lex(char('{')), lex(char('}')), many(arm()))
            .map(Vec::into)
            .expected("match body")
    };
    attempt(lex(keyword("match")))
        .with((expr(0), body()))
        .map(|(expr, arm)| Match {
            expr: Box::new(expr),
            arm,
        })
}
fn control_flow_<I>() -> impl Parser<I, Output = ControlFlow<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        block().map(ControlFlow::Block),
        if_expression().map(ControlFlow::If),
        for_expression().map(ControlFlow::For),
        while_expression().map(ControlFlow::While),
        loop_expression().map(ControlFlow::Loop),
        match_expression().map(ControlFlow::Match),
    ))
}
combine::parser! {
    pub(super) fn control_flow[I]()(I) -> ControlFlow<()>
    where [
        I: Stream<Token = char>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        control_flow_()
    }
}
