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
    ParseError, Parser, Stream,
};
use hir::{
    expr::{Block, ControlFlow, Expr, For, If, Match, MatchArm, While},
    statement::Statement,
};

#[derive(Debug, Clone, PartialEq)]
struct StatementExpr<T> {
    statement: Vec<Statement<T>>,
    expr: Option<Expr<T>>,
}
impl<T> Default for StatementExpr<T> {
    fn default() -> Self {
        Self {
            statement: Vec::new(),
            expr: None,
        }
    }
}
impl<T> Extend<StatementReturn<T>> for StatementExpr<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = StatementReturn<T>>,
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
pub(crate) fn block<I, T>() -> impl Parser<I, Output = Block<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
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
fn if_<I, T>() -> impl Parser<I, Output = If<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
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
    fn if_expression[I, T]()(I) -> If< T>
    where [
        I: Stream<Token = char>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        T: Default,
    ] {
        if_()
    }
}
fn for_expression<I, T>() -> impl Parser<I, Output = For<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    attempt(lex(keyword("for")))
        .with((pattern(), lex(keyword("in")), expr(0), block()))
        .map(|(pattern, _, expr, body)| For {
            pattern,
            expr: Box::new(expr),
            body,
        })
}
fn while_expression<I, T>() -> impl Parser<I, Output = While<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    attempt(lex(keyword("while")))
        .with((expr(0), block()))
        .map(|(condition, body)| While {
            condition: Box::new(condition),
            body,
        })
}
fn loop_expression<I, T>() -> impl Parser<I, Output = Block<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    attempt(lex(keyword("loop"))).with(block())
}
fn match_expression<I, T>() -> impl Parser<I, Output = Match<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    let arm_expr = || {
        choice((
            attempt(control_flow())
                .skip(optional(lex(char(','))))
                .map(Expr::ControlFlow),
            expr(0).skip(choice((
                lex(char(',')).map(|_| ()),
                look_ahead(char('}')).map(|_| ()),
            ))),
        ))
    };
    let arm = || {
        (pattern().skip(lex(string("=>"))), arm_expr()).map(|(pattern, expr)| MatchArm {
            pattern,
            expr: Box::new(expr),
        })
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
fn control_flow_<I, T>() -> impl Parser<I, Output = ControlFlow<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
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
    pub(crate) fn control_flow[I, T]()(I) -> ControlFlow< T>
    where [
        I: Stream<Token = char>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        T: Default,
    ] {
        control_flow_()
    }
}
