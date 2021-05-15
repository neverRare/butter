use crate::expr::control_flow::Block;
use crate::expr::control_flow::For;
use crate::expr::control_flow::If;
use crate::expr::control_flow::Match;
use crate::expr::control_flow::MatchArm;
use crate::expr::control_flow::While;
use crate::expr::Expr;
use crate::parser::expr::expr;
use crate::parser::ident_keyword::keyword;
use crate::parser::lex;
use crate::parser::pattern::pattern;
use crate::parser::statement::statement_return;
use crate::parser::statement::StatementReturn;
use crate::statement::Statement;
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

#[derive(Default, Debug, Clone, PartialEq)]
struct StatementExpr<'a> {
    statement: Vec<Statement<'a>>,
    expr: Option<Expr<'a>>,
}
impl<'a> Extend<StatementReturn<'a>> for StatementExpr<'a> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = StatementReturn<'a>>,
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
pub fn block<'a, I>() -> impl Parser<I, Output = Block<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
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
}
fn if_<'a, I>() -> impl Parser<I, Output = If<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
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
    fn if_expression['a, I]()(I) -> If<'a>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        if_()
    }
}
fn for_expression<'a, I>() -> impl Parser<I, Output = For<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
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
fn while_expression<'a, I>() -> impl Parser<I, Output = While<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    attempt(lex(keyword("while")))
        .with((expr(0), block()))
        .map(|(condition, body)| While {
            condition: Box::new(condition),
            body,
        })
}
fn loop_expression<'a, I>() -> impl Parser<I, Output = Block<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    attempt(lex(keyword("loop"))).with(block())
}
fn match_expression<'a, I>() -> impl Parser<I, Output = Match<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
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
fn control_flow_<'a, I>() -> impl Parser<I, Output = Expr<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
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
    pub fn control_flow['a, I]()(I) -> Expr<'a>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        control_flow_()
    }
}
