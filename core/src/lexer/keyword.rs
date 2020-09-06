use crate::lexer::Ident;
use util::lexer::Lex;
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Keyword {
    Abort,
    Move,
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
}
impl<'a> Lex<'a> for Keyword {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        let (move_state, Ident(ident)) = Lex::lex_first(src)?;
        let keyword = match ident {
            "abort" => Self::Abort,
            "move" => Self::Move,
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
            _ => return None,
        };
        Some((move_state, keyword))
    }
}
