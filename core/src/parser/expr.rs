use crate::ast::expr::Expr;
use crate::parser::expr::infix::infix_0;
use crate::parser::expr::infix::PartialAst;
use crate::parser::ident_keyword::ident;
use crate::parser::ident_keyword::keyword;
use crate::parser::lex;
use crate::parser::Parser;
use combine::attempt;
use combine::between;
use combine::choice;
use combine::many;
use combine::parser;
use combine::parser::char::char;
use combine::ParseError;
use combine::RangeStream;

mod infix;

fn prefix_expr<'a, I>() -> impl Parser<I, Output = Expr<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        between(lex(char('(')), lex(char(')')), expr(infix_0())),
        attempt(lex(ident())).map(Expr::Var),
        lex(keyword("false")).map(|_| Expr::False),
        lex(keyword("null")).map(|_| Expr::Null),
        lex(keyword("true")).map(|_| Expr::True),
    ))
}
parser! {
    pub fn expr['a, I, P](infix_parser: P)(I) -> Expr<'a>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        P: Parser<I, Output = PartialAst<'a>>,
    ] {
        (prefix_expr(), many(infix_parser)).map(|(prefix, infixes): (Expr, Vec<PartialAst>)| prefix)
    }
}
