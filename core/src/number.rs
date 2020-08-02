use crate::lexer::{LexerError, Num};

pub enum ParseResult {
    Ok(usize, Num),
    Err(LexerError, usize, usize),
    None,
}
pub fn parse_number(src: &str) -> ParseResult {
    todo!()
}
