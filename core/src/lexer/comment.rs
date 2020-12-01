use std::num::NonZeroUsize;
use util::lexer::Lex;

pub struct Comment;
impl<'a> Lex<'a> for Comment {
    fn lex_first(src: &'a str) -> Option<(NonZeroUsize, Self)> {
        if let Some("--") = src.get(..2) {
            match src[2..].find('\n') {
                None => Some((NonZeroUsize::new(src.len()).unwrap(), Self)),
                Some(i) => Some((NonZeroUsize::new(i + 3).unwrap(), Self)),
            }
        } else {
            None
        }
    }
}
