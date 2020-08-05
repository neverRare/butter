use crate::span::Span;

pub enum EscapeError {
    InvalidChar,
    InvalidHex,
}
pub enum StrError<'a> {
    InvalidEscape(Vec<(Span<'a>, EscapeError)>),
    Unterminated,
}
pub fn parse_string(quote: char, rest: &str) -> (usize, Result<Vec<u8>, Vec<StrError>>) {
    let mut vec = vec![];
    todo!()
}
