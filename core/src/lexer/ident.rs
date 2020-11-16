use util::lexer::Lex;

pub struct Ident;
impl<'a> Lex<'a> for Ident {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        let start = src.chars().next()?;
        if let '0'..='9' = start {
            None
        } else {
            match src.find(|ch: char| ch != '_' && !ch.is_alphanumeric()) {
                None => Some((src.len(), Self)),
                Some(0) => None,
                Some(i) => Some((i, Self)),
            }
        }
    }
}
