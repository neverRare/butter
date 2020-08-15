use crate::lexer::Bracket;
use crate::lexer::LexerError;
use crate::lexer::Token;
use crate::lexer::TokenSpans;
pub enum BaseTreeResult<'a> {
    Token(&'a str, Token<'a>),
    LexerError(&'a str, LexerError<'a>),
    TreeError(BracketError<'a>),
    In(Bracket),
    Out,
}
pub enum BracketError<'a> {
    Mismatch((&'a str, Bracket), (&'a str, Bracket)),
    Unmatched(&'a str, Bracket),
}
pub struct BaseTreeSpans<'a> {
    tokens: Option<TokenSpans<'a>>,
    closes: Vec<Bracket>,
}
impl<'a> BaseTreeSpans<'a> {
    pub fn new(src: &'a str) -> Self {
        src.into()
    }
}
impl<'a, T> From<T> for BaseTreeSpans<'a>
where
    T: Into<TokenSpans<'a>>,
{
    fn from(val: T) -> Self {
        BaseTreeSpans {
            tokens: Some(val.into()),
            closes: vec![],
        }
    }
}
impl<'a> Iterator for BaseTreeSpans<'a> {
    type Item = BaseTreeResult<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
