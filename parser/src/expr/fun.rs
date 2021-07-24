use crate::ident_keyword::ident;
use crate::lex;
use crate::pattern::parameter;
use combine::choice;
use combine::parser::char::string;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;
use hir::pattern::Pattern;
use hir::pattern::Var;
use std::collections::HashMap;
use std::iter::once;

pub fn param_arrow<'a, I, T>() -> impl Parser<I, Output = HashMap<&'a str, Pattern<'a, T>>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    let arrow = || lex(string("=>"));
    choice((
        arrow().map(|_| HashMap::new()),
        lex(ident())
            .map(|ident| {
                let mut map: HashMap<_, _> = once((
                    ident,
                    Pattern::Var(Var {
                        ident,
                        mutable: false,
                        bind_to_ref: false,
                        ty: T::default(),
                    }),
                ))
                .collect();
                map.shrink_to_fit();
                map
            })
            .skip(arrow()),
        parameter().skip(arrow()),
    ))
}
