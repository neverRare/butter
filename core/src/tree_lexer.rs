use crate::lexer::Bracket;
use crate::lexer::LexerError;
use crate::lexer::Opening;
use crate::lexer::Token;
use crate::lexer::TokenSpans;
pub enum BaseTreeResult<'a> {
    Token(&'a str, Token<'a>),
    LexerError(&'a str, LexerError<'a>),
    TreeError(BracketError<'a>),
    In(&'a str, Bracket),
    Out(&'a str),
}
pub enum BracketError<'a> {
    Mismatch((&'a str, Bracket), (&'a str, Bracket)),
    Unexpected(&'a str, Bracket),
    Unmatched(Vec<(&'a str, Bracket)>),
}
pub struct BaseTreeSpans<'a> {
    tokens: TokenSpans<'a>,
    closes: Vec<(&'a str, Bracket)>,
    done: bool,
}
impl<'a> BaseTreeSpans<'a> {
    pub fn new<T: Into<Self>>(src: T) -> Self {
        src.into()
    }
}
impl<'a, T> From<T> for BaseTreeSpans<'a>
where
    T: Into<TokenSpans<'a>>,
{
    fn from(val: T) -> Self {
        BaseTreeSpans {
            tokens: val.into(),
            closes: vec![],
            done: true,
        }
    }
}
impl<'a> Iterator for BaseTreeSpans<'a> {
    type Item = BaseTreeResult<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else if let Some((src, token)) = self.tokens.next() {
            let res = match token {
                Ok(Token::Bracket(Opening::Open, bracket)) => {
                    self.closes.push((src, bracket));
                    BaseTreeResult::In(src, bracket)
                }
                Ok(Token::Bracket(Opening::Close, bracket)) => match self.closes.pop() {
                    Some((_, open)) if open == bracket => BaseTreeResult::Out(src),
                    Some((first_src, open)) => {
                        self.done = true;
                        BaseTreeResult::TreeError(BracketError::Mismatch(
                            (first_src, open),
                            (src, bracket),
                        ))
                    }
                    None => {
                        self.done = true;
                        BaseTreeResult::TreeError(BracketError::Unexpected(src, bracket))
                    }
                },
                Ok(token) => BaseTreeResult::Token(src, token),
                Err(err) => BaseTreeResult::LexerError(src, err),
            };
            Some(res)
        } else if self.closes.is_empty() {
            self.done = true;
            None
        } else {
            self.done = true;
            let errs = self.closes.drain(..).collect();
            Some(BaseTreeResult::TreeError(BracketError::Unmatched(errs)))
        }
    }
}
