use crate::lexer::Token;
pub struct TokenTree<'a> {
    token: Token<'a>,
    tree: Vec<TokenTree<'a>>,
}
impl<'a> TokenTree<'a> {
    pub fn from_tokens(tokens: Vec<Token<'a>>) -> Vec<Self> {
        todo!()
    }
    pub fn lex(src: &'a str) -> Vec<Self> {
        Self::from_tokens(Token::lex(src))
    }
}
