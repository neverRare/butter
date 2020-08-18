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
    Out(&'a str, Bracket),
}
pub enum BracketError<'a> {
    Mismatch((&'a str, Bracket), (&'a str, Bracket)),
    Unexpected(&'a str, Bracket),
    Unmatched(&'a str, Bracket),
}
pub struct BaseTreeSpans<'a> {
    tokens: TokenSpans<'a>,
    closes: Vec<(&'a str, Bracket)>,
    done: bool,
    err: bool,
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
            done: false,
            err: false,
        }
    }
}
impl<'a> Iterator for BaseTreeSpans<'a> {
    type Item = BaseTreeResult<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else if self.err {
            if let Some((src, bracket)) = self.closes.pop() {
                Some(BaseTreeResult::TreeError(BracketError::Unmatched(
                    src, bracket,
                )))
            } else {
                self.done = true;
                None
            }
        } else if let Some((src, token)) = self.tokens.next() {
            let res = match token {
                Ok(Token::Bracket(Opening::Open, bracket)) => {
                    self.closes.push((src, bracket));
                    BaseTreeResult::In(src, bracket)
                }
                Ok(Token::Bracket(Opening::Close, bracket)) => match self.closes.pop() {
                    Some((_, open)) if open == bracket => BaseTreeResult::Out(src, bracket),
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
        } else if let Some((src, bracket)) = self.closes.pop() {
            self.err = true;
            Some(BaseTreeResult::TreeError(BracketError::Unmatched(
                src, bracket,
            )))
        } else {
            self.done = true;
            None
        }
    }
}
pub enum TokenTreeError<'a> {
    Token(&'a str, LexerError<'a>),
    Tree(BracketError<'a>),
}
pub type TokenTrees<'a> = Vec<TokenTree<'a>>;
pub enum TokenTree<'a> {
    Token(Token<'a>),
    Tree(Bracket, TokenTrees<'a>),
}
impl<'a> TokenTree<'a> {
    pub fn lex(src: &'a str) -> Result<Vec<Self>, Vec<TokenTreeError<'a>>> {
        let mut errors = vec![];
        let mut stack: Vec<Vec<Self>> = vec![];
        let mut current: Vec<Self> = vec![];
        for token in BaseTreeSpans::new(src) {
            match token {
                BaseTreeResult::Token(_, token) => {
                    if errors.is_empty() {
                        current.push(TokenTree::Token(token));
                    }
                }
                BaseTreeResult::In(_, _) => {
                    if errors.is_empty() {
                        stack.push(current.drain(..).collect());
                    }
                }
                BaseTreeResult::Out(_, bracket) => {
                    if errors.is_empty() {
                        let prev = stack.pop().unwrap();
                        current.push(TokenTree::Tree(bracket, prev));
                    }
                }
                BaseTreeResult::LexerError(src, err) => {
                    errors.push(TokenTreeError::Token(src, err));
                }
                BaseTreeResult::TreeError(err) => {
                    errors.push(TokenTreeError::Tree(err));
                }
            }
        }
        if errors.is_empty() {
            Err(errors)
        } else {
            Ok(current)
        }
    }
}
