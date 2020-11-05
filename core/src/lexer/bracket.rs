use util::lexer::Lex;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Bracket {
    Parenthesis,
    Bracket,
    Brace,
}
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Opening {
    Open,
    Close,
}
pub struct OpeningBracket(pub Opening, pub Bracket);
impl<'a> Lex<'a> for OpeningBracket {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        let (opening, bracket) = match src.get(..1)? {
            "(" => (Opening::Open, Bracket::Parenthesis),
            ")" => (Opening::Close, Bracket::Parenthesis),
            "[" => (Opening::Open, Bracket::Bracket),
            "]" => (Opening::Close, Bracket::Bracket),
            "{" => (Opening::Open, Bracket::Brace),
            "}" => (Opening::Close, Bracket::Brace),
            _ => return None,
        };
        Some((1, Self(opening, bracket)))
    }
}
