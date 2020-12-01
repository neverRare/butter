use std::num::NonZeroUsize;
use util::lexer::Lex;

pub struct Ident;
impl<'a> Lex<'a> for Ident {
    fn lex_first(src: &'a str) -> Option<(NonZeroUsize, Self)> {
        let start = src.chars().next()?;
        if let '0'..='9' = start {
            None
        } else {
            match src.find(|ch: char| ch != '_' && !ch.is_alphanumeric()) {
                None => Some((NonZeroUsize::new(src.len()).unwrap(), Self)),
                Some(i) => NonZeroUsize::new(i).map(|i| (i, Self)),
            }
        }
    }
}
