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
impl<'a> From<TokenSpans<'a>> for TokenTreeSpans<'a> {
    fn from(val: TokenSpans<'a>) -> Self {
        Self {
            tokens: Some(val),
            closes: vec![],
        }
    }
}
impl<'a> From<&'a str> for TokenTreeSpans<'a> {
    fn from(val: &'a str) -> Self {
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
