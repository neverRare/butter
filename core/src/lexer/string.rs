#[derive(PartialEq, Eq, Debug)]
pub enum EscapeError {
    InvalidChar,
    InvalidHex,
}
#[derive(PartialEq, Eq, Debug)]
pub enum StrError<'a> {
    InvalidEscape(Vec<(&'a str, EscapeError)>),
    Unterminated,
}
pub fn parse_string(quote: char, rest: &str) -> (usize, Result<Vec<u8>, StrError>) {
    todo!()
}
