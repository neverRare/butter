use util::lexer::Lex;

pub struct Comment<'a>(pub &'a str);
impl<'a> Lex<'a> for Comment<'a> {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        if let Some("--") = src.get(..2) {
            match src[2..].find('\n') {
                None => Some((src.len(), Self(&src[2..]))),
                Some(0) => None,
                Some(i) => Some((i + 3, Self(&src[2..i + 2]))),
            }
        } else {
            None
        }
    }
}
