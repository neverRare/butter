use util::lexer::Lex;
use util::lexer::MoveState;

enum Char<'a> {
    Char(char),
    Byte(u8),
    Quote,
    DoubleQuote,
    Error(&'a str, EscapeError),
}
impl<'a> Lex<'a> for Char<'a> {
    fn lex_first(src: &'a str) -> Option<(MoveState, Self)> {
        let first = src.chars().next().unwrap();
        if let '\\' = first {
            let escape = src[1..].chars().next()?;
            if let 'x' = escape {
                let mut len = 0;
                let mut codes: Vec<u8> = vec![];
                let code_rest = &src[2..];
                let code_chars = code_rest
                    .char_indices()
                    .take(2)
                    .take_while(|(_, ch)| *ch != first && *ch != '\\');
                for (ind, ch) in code_chars {
                    len = ind + ch.len_utf8();
                    match ch.to_digit(16) {
                        Some(val) => codes.push(val as u8),
                        None => {
                            return Some((
                                MoveState::Move(2 + len),
                                Self::Error(
                                    &code_rest[ind..ind + ch.len_utf8()],
                                    EscapeError::InvalidHexChar,
                                ),
                            ))
                        }
                    }
                }
                let token = if codes.len() != 2 {
                    let slice = if codes.is_empty() {
                        &src[1..2]
                    } else {
                        &code_rest[..len]
                    };
                    Self::Error(slice, EscapeError::HexCharTerminated)
                } else {
                    Self::Byte((codes[0] << 4) + codes[1])
                };
                Some((MoveState::Move(2 + len), token))
            } else {
                let byte = match escape {
                    '\\' => b'\\',
                    '\'' => b'\'',
                    '"' => b'"',
                    'n' => b'\n',
                    'r' => b'\r',
                    't' => b'\t',
                    'v' => b'\x30',
                    '0' => b'\0',
                    _ => {
                        return Some((
                            MoveState::Move(1 + escape.len_utf8()),
                            Self::Error(&src[1..1 + escape.len_utf8()], EscapeError::InvalidChar),
                        ))
                    }
                };
                Some((MoveState::Move(2), Self::Byte(byte)))
            }
        } else if let '\'' = first {
            Some((MoveState::Move(1), Self::Quote))
        } else if let '"' = first {
            Some((MoveState::Move(1), Self::DoubleQuote))
        } else {
            Some((MoveState::Move(1), Self::Char(first)))
        }
    }
}
pub enum StrLiteral<'a> {
    Char(u8),
    Str(Vec<u8>),
    Error(StrError<'a>),
}
#[derive(PartialEq, Eq, Debug)]
pub enum EscapeError {
    InvalidChar,
    InvalidHexChar,
    HexCharTerminated,
}
#[derive(PartialEq, Eq, Debug)]
pub enum StrError<'a> {
    InvalidEscape(Vec<(&'a str, EscapeError)>),
    Unterminated,
    CharNotOne,
}