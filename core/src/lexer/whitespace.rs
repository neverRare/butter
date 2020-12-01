use std::num::NonZeroUsize;
use util::lexer::Lex;

pub struct Whitespace;
impl<'a> Lex<'a> for Whitespace {
    fn lex_first(src: &'a str) -> Option<(NonZeroUsize, Self)> {
        match src.find(|ch: char| !ch.is_whitespace()) {
            None => Some((NonZeroUsize::new(src.len()).unwrap(), Self)),
            Some(i) => NonZeroUsize::new(i).map(|i| (i, Self)),
        }
    }
}
