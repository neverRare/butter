use crate::ast::expr::control_flow::Block;
use crate::ast::expr::control_flow::For;
use crate::ast::expr::control_flow::If;
use crate::ast::expr::control_flow::While;
use crate::ast::expr::Expr;
use crate::ast::statement::Statement;
use crate::parser::expr::expr;
use crate::parser::ident_keyword::keyword;
use crate::parser::lex;
use crate::parser::pattern::pattern;
use crate::parser::statement::statement_return;
use crate::parser::statement::StatementReturn;
use combine::attempt;
use combine::between;
use combine::choice;
use combine::many;
use combine::optional;
use combine::parser;
use combine::parser::char::char;
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
fn block<'a, I>() -> impl Parser<I, Output = Block<'a>>
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
            statement,
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
pub fn control_flow<'a, I>() -> impl Parser<I, Output = Expr<'a>>
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
    ))
}
