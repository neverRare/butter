use crate::ast::expr::control_flow::Break;
use crate::ast::expr::Expr;
use crate::parser::expr::array::range;
use crate::parser::expr::infix::infix_0;
use crate::parser::expr::infix::infix_7;
use crate::parser::expr::infix::PartialAst;
use crate::parser::expr::record::record;
use crate::parser::ident_keyword::ident;
use crate::parser::ident_keyword::ident_or_keyword;
use crate::parser::ident_keyword::keyword;
use crate::parser::lex;
use crate::parser::Parser;
use combine::attempt;
use combine::between;
use combine::choice;
use combine::error::StreamError;
use combine::many;
use combine::optional;
use combine::parser;
use combine::parser::char::char;
use combine::stream::StreamErrorFor;
use combine::ParseError;
use combine::RangeStream;

mod array;
mod infix;
mod record;

fn prefix_expr<'a, I>() -> impl Parser<I, Output = Expr<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        range().map(Expr::ArrayRange),
        attempt(between(lex(char('(')), lex(char(')')), expr(infix_0()))),
        record().map(Expr::Struct),
        (lex(char('!')), expr(infix_7())).map(|(_, expr)| Expr::Not(Box::new(expr))),
        (lex(char('&')), expr(infix_7())).map(|(_, expr)| Expr::Ref(Box::new(expr))),
        (lex(char('+')), expr(infix_7())).map(|(_, expr)| Expr::Plus(Box::new(expr))),
        (lex(char('-')), expr(infix_7())).map(|(_, expr)| Expr::Minus(Box::new(expr))),
        attempt(lex(ident())).map(Expr::Var),
        (attempt(lex(keyword("clone"))), expr(infix_7()))
            .map(|(_, expr)| Expr::Clone(Box::new(expr))),
        lex(keyword("false")).map(|_| Expr::False),
        lex(keyword("null")).map(|_| Expr::Null),
        lex(keyword("true")).map(|_| Expr::True),
        (
            lex(keyword("break")),
            optional(lex(ident_or_keyword())),
            optional((lex(char('=')), expr(infix_0()))),
        )
            .map(|(_, label, expr)| {
                Expr::Break(Break {
                    label,
                    expr: expr.map(|(_, expr)| Box::new(expr)),
                })
            }),
        (lex(keyword("continue")), optional(lex(ident_or_keyword())))
            .map(|(_, label)| Expr::Continue(label)),
        (lex(keyword("return")), optional(expr(infix_0())))
            .map(|(_, expr)| Expr::Return(expr.map(Box::new))),
    ))
}
parser! {
    pub fn expr['a, I, P](infix_parser: P)(I) -> Expr<'a>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        P: Parser<I, Output = PartialAst<'a>>,
    ] {
        (prefix_expr(), many(infix_parser)).and_then(|(prefix, infixes)| {
            let mut prefix = prefix;
            let infixes: Vec<_> = infixes;
            for infix in infixes {
                prefix = match infix.combine_from(prefix) {
                    Some(expr) => expr,
                    None => {
                        return Err(<StreamErrorFor<I>>::unexpected_static_message(
                            "non place expression",
                        ))
                    }
                };
            }
            Ok(prefix)
        })
    }
}
