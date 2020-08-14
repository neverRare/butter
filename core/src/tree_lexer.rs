use crate::lexer::Bracket;
use crate::lexer::LexerError;
use crate::lexer::Token;
use crate::lexer::TokenSpans;
pub enum TreeResult<'a> {
    Token(&'a str, Token<'a>),
    LexerError(&'a str, LexerError<'a>),
    TreeError,
    In(Bracket, TokenTreeSpans<'a>),
    Out(TokenTreeSpans<'a>),
}
pub enum TreeError<'a> {
    Mismatch((&'a str, Bracket), (&'a str, Bracket)),
    Unmatched(Vec<(&'a str, Bracket)>),
}
pub struct TokenTreeSpans<'a> {
    tokens: Option<TokenSpans<'a>>,
    closes: Vec<Bracket>,
}
impl<'a> TokenTreeSpans<'a> {
    pub fn new(src: &'a str) -> Self {
        src.into()
    }
}
impl<'a, T> From<T> for TokenTreeSpans<'a>
where
    T: Into<TokenSpans<'a>>,
{
    fn from(val: T) -> Self {
        Self {
            tokens: Some(val.into()),
            closes: vec![],
        }
    }
}
impl<'a> Iterator for TokenTreeSpans<'a> {
    type Item = TreeResult<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
