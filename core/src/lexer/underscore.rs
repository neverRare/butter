use crate::lexer::Ident;
use std::num::NonZeroUsize;
use util::lexer::Lex;

pub struct Underscore;
impl<'a> Lex<'a> for Underscore {
    fn lex_first(src: &'a str) -> Option<(NonZeroUsize, Self)> {
        if let ("_", Ident) = Lex::lex_first_span(src)? {
            Some((NonZeroUsize::new(1).unwrap(), Self))
        } else {
            None
        }
    }
}
