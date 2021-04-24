use crate::expr::control_flow::Param;
use crate::parser::ident_keyword::ident;
use crate::parser::lex;
use crate::parser::pattern::parameter;
use crate::pattern::Pattern;
use combine::choice;
use combine::parser::char::string;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;
use std::iter::once;

pub fn param_arrow<'a, I>() -> impl Parser<I, Output = Param<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let arrow = || lex(string("=>"));
    choice((
        arrow().map(|_| Param::default()),
        lex(ident())
            .map(|ident| Param {
                order: vec![ident],
                param: once((ident, Pattern::Var(ident))).collect(),
            })
            .skip(arrow()),
        parameter().skip(arrow()),
    ))
}
