use crate::lexer::number::InvalidChar;
use crate::lexer::number::Num;
use crate::lexer::number::NumError;
use crate::lexer::string::EscapeError;
use crate::lexer::string::StrError;
use number::parse_number;
use std::num::NonZeroUsize;
use string::parse_string;
use util::lexer::Lex;
use util::lexer::MoveState;

mod number;
mod string;

struct Ident<'a>(&'a str);
impl<'a> Lex<'a> for Ident<'a> {
    fn lex_first(src: &'a str) -> Option<(MoveState, Self)> {
        match src.find(|ch: char| !ch.is_alphanumeric()) {
            None => Some((MoveState::Stop, Self(src))),
            Some(0) => None,
            Some(i) => Some((
                MoveState::Move(NonZeroUsize::new(i).unwrap()),
                Self(&src[..i]),
            )),
        }
    }
}
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Bracket {
    Paren,
    Bracket,
    Brace,
}
#[derive(PartialEq, Eq, Debug)]
pub enum Opening {
    Open,
    Close,
}
struct OpeningBracket(Opening, Bracket);
impl<'a> Lex<'a> for OpeningBracket {
    fn lex_first(src: &'a str) -> Option<(MoveState, Self)> {
        let (opening, bracket) = match src.get(..1)? {
            "(" => (Opening::Open, Bracket::Paren),
            ")" => (Opening::Close, Bracket::Paren),
            "[" => (Opening::Open, Bracket::Bracket),
            "]" => (Opening::Close, Bracket::Bracket),
            "{" => (Opening::Open, Bracket::Brace),
            "}" => (Opening::Close, Bracket::Brace),
            _ => return None,
        };
        Some((
            MoveState::Move(NonZeroUsize::new(1).unwrap()),
            Self(opening, bracket),
        ))
    }
}
#[derive(PartialEq, Eq, Debug)]
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
    fn lex_first(src: &'a str) -> Option<(MoveState, Self)> {
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
#[derive(PartialEq, Eq, Debug)]
pub enum Separator {
    Comma,
    Semicolon,
}
impl<'a> Lex<'a> for Separator {
    fn lex_first(src: &'a str) -> Option<(MoveState, Self)> {
        let separator = match src.get(..1)? {
            "," => Self::Comma,
            ";" => Self::Semicolon,
            _ => return None,
        };
        Some((MoveState::Move(NonZeroUsize::new(1).unwrap()), separator))
    }
}
#[derive(PartialEq, Eq, Debug)]
pub enum Operator {
    Equal,
    DoubleEqual,
    NotEqual,
    Colon,
    DoubleColon,
    Dot,
    DoubleDot,
    Plus,
    Minus,
    Star,
    Slash,
    DoubleSlash,
    Percent,
    Bang,
    Amp,
    Pipe,
    Caret,
    Tilde,
    DoubleAmp,
    DoublePipe,
    Greater,
    Less,
    DoubleGreater,
    DoubleLess,
    GreaterEqual,
    LessEqual,
    LeftArrow,
    RightArrow,
    RightThickArrow,
    Question,
    DoubleQuestion,
}
impl<'a> Lex<'a> for Operator {
    fn lex_first(src: &'a str) -> Option<(MoveState, Self)> {
        let special = src
            .get(..3)
            .map(|val| val == "<--" || val == "==>")
            .unwrap_or(false);
        if !special {
            let operator = src.get(..2).and_then(|operator| match operator {
                "==" => Some(Self::DoubleEqual),
                "!=" => Some(Self::NotEqual),
                "::" => Some(Self::DoubleColon),
                ".." => Some(Self::DoubleDot),
                "//" => Some(Self::DoubleSlash),
                "&&" => Some(Self::DoubleAmp),
                "||" => Some(Self::DoublePipe),
                ">>" => Some(Self::DoubleGreater),
                "<<" => Some(Self::DoubleLess),
                ">=" => Some(Self::GreaterEqual),
                "<=" => Some(Self::LessEqual),
                "<-" => Some(Self::LeftArrow),
                "->" => Some(Self::RightArrow),
                "=>" => Some(Self::RightThickArrow),
                "??" => Some(Self::DoubleQuestion),
                _ => None,
            });
            if let Some(operator) = operator {
                return Some((MoveState::Move(NonZeroUsize::new(2).unwrap()), operator));
            }
        }
        let operator = src.get(..1)?;
        let operator = match operator {
            "=" => Self::Equal,
            ":" => Self::Colon,
            "." => Self::Dot,
            "+" => Self::Plus,
            "-" => Self::Minus,
            "*" => Self::Star,
            "/" => Self::Slash,
            "%" => Self::Percent,
            "!" => Self::Bang,
            "&" => Self::Amp,
            "|" => Self::Pipe,
            "^" => Self::Caret,
            "~" => Self::Tilde,
            ">" => Self::Greater,
            "<" => Self::Less,
            "?" => Self::Question,
            _ => return None,
        };
        Some((MoveState::Move(NonZeroUsize::new(1).unwrap()), operator))
    }
}
#[derive(PartialEq, Debug)]
pub enum Token<'a> {
    Num(Num),
    Str(Vec<u8>),
    Char(u8),
    Keyword(Keyword),
    Identifier(&'a str),
    Separator(Separator),
    Bracket(Opening, Bracket),
    Operator(Operator),
    Error(LexerError<'a>),
}
#[derive(PartialEq, Eq, Debug)]
pub enum LexerError<'a> {
    UnknownChar,
    UnterminatedQuote(char),
    InvalidEscape(Vec<(&'a str, EscapeError)>),
    CharNotOne,
    InvalidChar(Vec<(&'a str, InvalidChar)>),
    Overflow,
}
struct TokenSpan<'a>(&'a str, Token<'a>);
// #[cfg(test)]
// mod test {
//     use super::{Bracket, Keyword, Num, Opening, Operator, Separator, Token};
//     #[test]
//     fn simple_lex() {
//         assert_eq!(
//             Token::lex("-- comment\n identifier true_false null => + ( ) ; <--"),
//             Ok(vec![
//                 Token::Identifier("identifier"),
//                 Token::Identifier("true_false"),
//                 Token::Keyword(Keyword::Null),
//                 Token::Operator(Operator::RightThickArrow),
//                 Token::Operator(Operator::Plus),
//                 Token::Bracket(Opening::Open, Bracket::Paren),
//                 Token::Bracket(Opening::Close, Bracket::Paren),
//                 Token::Separator(Separator::Semicolon),
//                 Token::Operator(Operator::Less),
//             ]),
//         );
//     }
//     #[test]
//     fn lex_string() {
//         assert_eq!(
//             Token::lex(
//                 r#"
// "hello world"
// "hello \"world\""
// "hello world \\"
// 'a'
// '\''
// '\\'
// '\x7A'
// """"
// 'a''a'
// "#
//             ),
//             Ok(vec![
//                 Token::Str(b"hello world".to_vec()),
//                 Token::Str(b"hello \"world\"".to_vec()),
//                 Token::Str(b"hello world \\".to_vec()),
//                 Token::Char(b'a'),
//                 Token::Char(b'\''),
//                 Token::Char(b'\\'),
//                 Token::Char(b'\x7A'),
//                 Token::Str(vec![]),
//                 Token::Str(vec![]),
//                 Token::Char(b'a'),
//                 Token::Char(b'a'),
//             ]),
//         );
//     }
//     #[test]
//     fn lex_number() {
//         assert_eq!(
//             Token::lex(
//                 r#"
// 12
// 0.5
// 0xff
// 0b11110000
// 0o127
// 1_000_000
// 4e-7
// 4e7
// 4e70
// 2.
// .5
// "#
//             ),
//             Ok(vec![
//                 Num::UInt(12),
//                 Num::Float(0.5),
//                 Num::UInt(0xff),
//                 Num::UInt(0b11110000),
//                 Num::UInt(0o127),
//                 Num::UInt(1_000_000),
//                 Num::Float(4e-7),
//                 Num::UInt(40_000_000),
//                 Num::Float(4e70),
//                 Num::UInt(2),
//             ]
//             .into_iter()
//             .map(Token::Num)
//             .chain(vec![
//                 Token::Operator(Operator::Dot),
//                 Token::Num(Num::Float(0.5)),
//             ])
//             .collect()),
//         );
//     }
// }
