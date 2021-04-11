use crate::ast::expr::control_flow::Break;
use crate::ast::expr::control_flow::Fun;
use crate::ast::expr::Expr;
use crate::parser::expr::array::array;
use crate::parser::expr::array::range;
use crate::parser::expr::control_flow::control_flow;
use crate::parser::expr::fun::param_arrow;
use crate::parser::expr::infix::infix;
use crate::parser::expr::integer::based_integer;
use crate::parser::expr::record::record;
use crate::parser::expr::string::char_literal;
use crate::parser::expr::string::string_literal;
use crate::parser::ident_keyword::ident;
use crate::parser::ident_keyword::ident_or_keyword;
use crate::parser::ident_keyword::keyword;
use crate::parser::lex;
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
use combine::Parser;
use combine::RangeStream;

mod array;
pub mod control_flow;
mod float;
mod fun;
mod infix;
mod integer;
mod record;
mod string;

fn prefix_expr_<'a, I>() -> impl Parser<I, Output = Expr<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        (attempt(param_arrow()), expr(0)).map(|(param, body)| {
            Expr::Fun(Fun {
                param,
                body: Box::new(body),
            })
        }),
        attempt(range()).map(Expr::ArrayRange),
        array().map(Expr::Array),
        attempt(between(lex(char('(')), lex(char(')')), expr(0))),
        record().map(Expr::Struct),
        lex(char_literal()).map(Expr::Char),
        lex(string_literal()).map(Expr::Str),
        lex(char('!'))
            .with(expr(7))
            .map(|expr| Expr::Not(Box::new(expr))),
        lex(char('&'))
            .with(expr(7))
            .map(|expr| Expr::Ref(Box::new(expr))),
        lex(char('+'))
            .with(expr(7))
            .map(|expr| Expr::Plus(Box::new(expr))),
        lex(char('-'))
            .with(expr(7))
            .map(|expr| Expr::Minus(Box::new(expr))),
        attempt(lex(ident())).map(Expr::Var),
        attempt(lex(keyword("clone")))
            .with(expr(7))
            .map(|expr| Expr::Clone(Box::new(expr))),
        control_flow(),
        lex(keyword("false")).map(|_| Expr::False),
        lex(keyword("null")).map(|_| Expr::Null),
        lex(keyword("true")).map(|_| Expr::True),
        lex(keyword("break"))
            .with((
                optional(lex(ident_or_keyword())),
                optional(lex(char('=')).with(expr(0))),
            ))
            .map(|(label, expr)| {
                Expr::Break(Break {
                    label,
                    expr: expr.map(Box::new),
                })
            }),
        lex(keyword("continue"))
            .with(optional(lex(ident_or_keyword())))
            .map(Expr::Continue),
        lex(keyword("return"))
            .with(optional(expr(0)))
            .map(|expr| Expr::Return(expr.map(Box::new))),
        lex(float::float()).map(Expr::Float),
        lex(based_integer()).map(Expr::UInt),
    ))
}
parser! {
    fn prefix_expr['a, I]()(I) -> Expr<'a>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        prefix_expr_()
    }
}
fn expr_<'a, I>(precedence: u8) -> impl Parser<I, Output = Expr<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (prefix_expr(), many(infix(precedence))).and_then(|(prefix, infixes)| {
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
parser! {
    pub fn expr['a, I](precedence: u8)(I) -> Expr<'a>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        expr_(*precedence)
    }
}
#[cfg(test)]
mod test {
    use crate::ast::expr::operator::Binary;
    use crate::parser::expr::expr;
    use crate::parser::expr::Expr;
    use combine::EasyParser;

    #[test]
    fn group() {
        let src = "(foo)";
        let expected = Expr::Var("foo");
        assert_eq!(expr(0).easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn precedence() {
        let src = "foo + bar * baz";
        let expected = Expr::Add(Binary {
            left: Box::new(Expr::Var("foo")),
            right: Box::new(Expr::Multiply(Binary {
                left: Box::new(Expr::Var("bar")),
                right: Box::new(Expr::Var("baz")),
            })),
        });
        assert_eq!(expr(0).easy_parse(src), Ok((expected, "")));
        let src = "foo * bar + baz";
        let expected = Expr::Add(Binary {
            left: Box::new(Expr::Multiply(Binary {
                left: Box::new(Expr::Var("foo")),
                right: Box::new(Expr::Var("bar")),
            })),
            right: Box::new(Expr::Var("baz")),
        });
        assert_eq!(expr(0).easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn ignore_higher_precedence() {
        let src = "foo + bar";
        let expected = Expr::Var("foo");
        let left = "+ bar";
        assert_eq!(expr(7).easy_parse(src), Ok((expected, left)));
    }
}
