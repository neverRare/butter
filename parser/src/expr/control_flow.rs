use crate::expr::expr;
use crate::ident_keyword::keyword;
use crate::lex;
use crate::pattern::pattern;
use crate::statement::statement_return;
use crate::statement::StatementReturn;
use combine::attempt;
use combine::between;
use combine::choice;
use combine::look_ahead;
use combine::many;
use combine::optional;
use combine::parser;
use combine::parser::char::char;
use combine::parser::char::string;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;
use hir::expr::control_flow::Block;
use hir::expr::control_flow::For;
use hir::expr::control_flow::If;
use hir::expr::control_flow::Match;
use hir::expr::control_flow::MatchArm;
use hir::expr::control_flow::While;
use hir::expr::Expr;
use hir::statement::Statement;

#[derive(Debug, Clone, PartialEq)]
struct StatementExpr<'a, T> {
    statement: Vec<Statement<'a, T>>,
    expr: Option<Expr<'a, T>>,
}
impl<'a, T> Default for StatementExpr<'a, T> {
    fn default() -> Self {
        Self {
            statement: Vec::new(),
            expr: None,
        }
    }
}
impl<'a, T> Extend<StatementReturn<'a, T>> for StatementExpr<'a, T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = StatementReturn<'a, T>>,
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
pub fn block<'a, I, T>() -> impl Parser<I, Output = Block<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
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
}
fn if_<'a, I, T>() -> impl Parser<I, Output = If<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    let else_part = || {
        lex(keyword("else")).with(choice((
            block().map(Expr::Block),
            if_expression().map(Expr::If),
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
parser! {
    fn if_expression['a, I, T]()(I) -> If<'a, T>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        T: Default,
    ] {
        if_()
    }
}
fn for_expression<'a, I, T>() -> impl Parser<I, Output = For<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
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
fn while_expression<'a, I, T>() -> impl Parser<I, Output = While<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
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
fn loop_expression<'a, I, T>() -> impl Parser<I, Output = Block<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    attempt(lex(keyword("loop"))).with(block())
}
fn match_expression<'a, I, T>() -> impl Parser<I, Output = Match<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    let arm_expr = || {
        choice((
            attempt(control_flow()).skip(optional(lex(char(',')))),
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
    let body = || between(lex(char('{')), lex(char('}')), many(arm())).map(Vec::into);
    attempt(lex(keyword("match")))
        .with((expr(0), body()))
        .map(|(expr, arm)| Match {
            expr: Box::new(expr),
            arm,
        })
}
fn control_flow_<'a, I, T>() -> impl Parser<I, Output = Expr<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    choice((
        block().map(Expr::Block),
        if_expression().map(Expr::If),
        for_expression().map(Expr::For),
        while_expression().map(Expr::While),
        loop_expression().map(Expr::Loop),
        match_expression().map(Expr::Match),
    ))
}
parser! {
    pub fn control_flow['a, I, T]()(I) -> Expr<'a, T>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        T: Default,
    ] {
        control_flow_()
    }
}
