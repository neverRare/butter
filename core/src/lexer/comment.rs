use util::lexer::Lex;

pub struct Comment;
impl<'a> Lex<'a> for Comment {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        if let Some("--") = src.get(..2) {
            match src[2..].find('\n') {
                None => Some((src.len(), Self)),
                Some(i) => Some((i + 3, Self)),
            }
        } else {
            None
        }
    }
}
