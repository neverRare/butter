use util::lexer::Lex;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Separator {
    Comma,
    Semicolon,
}
impl<'a> Lex<'a> for Separator {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        let separator = match src.get(..1)? {
            "," => Self::Comma,
            ";" => Self::Semicolon,
            _ => return None,
        };
        Some((1, separator))
    }
}
