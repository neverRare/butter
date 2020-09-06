use util::lexer::Lex;

pub struct Whitespace;
impl<'a> Lex<'a> for Whitespace {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        match src.find(|ch: char| !ch.is_whitespace()) {
            None => Some((src.len(), Self)),
            Some(0) => None,
            Some(i) => Some((i, Self)),
        }
    }
}
