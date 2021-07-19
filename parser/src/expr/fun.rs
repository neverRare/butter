use crate::ident_keyword::ident;
use crate::lex;
use crate::pattern::parameter;
use combine::choice;
use combine::parser::char::string;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;
use hir::expr::control_flow::Param;
use hir::pattern::Pattern;
use hir::pattern::Var;
use std::collections::HashMap;
use std::iter::once;

pub fn param_arrow<'a, I>() -> impl Parser<I, Output = Param<'a, ()>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let arrow = || lex(string("=>"));
    choice((
        arrow().map(|_| Param::default()),
        lex(ident())
            .map(|ident| Param {
                order: vec![ident].into(),
                param: {
                    let mut map: HashMap<_, _> = once((
                        ident,
                        Pattern::Var(Var {
                            ident,
                            mutable: false,
                            bind_to_ref: false,
                            ty: (),
                        }),
                    ))
                    .collect();
                    map.shrink_to_fit();
                    map
                },
            })
            .skip(arrow()),
        parameter().skip(arrow()),
    ))
}
