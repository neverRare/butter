
use crate::lexer::Ident;
use util::lexer::Lex;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Keyword {
    True,
    False,
    Null,
    Clone,
    If,
    Else,
    For,
    In,
    Loop,
    While,
    Break,
    Continue,
    Return,
}
impl<'a> Lex<'a> for Keyword {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        let (ident, Ident) = Lex::lex_first_span(src)?;
        let keyword = match ident {
            "true" => Self::True,
            "false" => Self::False,
            "null" => Self::Null,
            "clone" => Self::Clone,
            "if" => Self::If,
            "else" => Self::Else,
            "for" => Self::For,
            "in" => Self::In,
            "loop" => Self::Loop,
            "while" => Self::While,
            "break" => Self::Break,
            "continue" => Self::Continue,
            "return" => Self::Return,
            _ => return None,
        };
        Some((ident.len(), keyword))
    }
}
