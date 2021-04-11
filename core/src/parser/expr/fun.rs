use crate::ast::pattern::Pattern;
use crate::parser::ident_keyword::ident;
use crate::parser::lex;
use crate::parser::pattern::parameter;
use combine::choice;
use combine::parser::char::string;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;
use std::collections::HashMap;
use std::iter::once;

pub fn param_arrow<'a, I>() -> impl Parser<I, Output = HashMap<&'a str, Pattern<'a>>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let arrow = || lex(string("=>"));
    choice((
        arrow().map(|_| HashMap::new()),
        lex(ident())
            .map(|ident| once((ident, Pattern::Var(ident))).collect())
            .skip(arrow()),
        parameter().skip(arrow()),
    ))
}
