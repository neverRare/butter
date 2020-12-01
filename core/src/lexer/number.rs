use std::num::NonZeroUsize;
use util::lexer::Lex;

pub struct Num;
impl<'a> Lex<'a> for Num {
    fn lex_first(src: &'a str) -> Option<(NonZeroUsize, Self)> {
        if let Some('0'..='9') = src.chars().next() {
            let pos = src[1..]
                .find(|ch: char| ch != '_' && !ch.is_alphanumeric())
                .unwrap_or(0);
            Some((NonZeroUsize::new(pos + 1).unwrap(), Self))
        } else {
            None
        }
    }
}
