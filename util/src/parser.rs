use crate::tree_vec::Tree;

// TODO: better error handling
trait Parser<'a>: Sized {
    type Token;
    type Tokens: Iterator<Item = Self::Token> + From<&'a str>;
    fn error_node() -> Self;
    fn prefix_parse(prefix: Self::Token, tokens: &mut Self::Tokens) -> Tree<Self>;
    fn infix_parse(infix: Self::Token, tokens: &mut Self::Tokens) -> Tree<Self>;
    fn prefix_precedence(token: &Self::Token) -> Option<u32>;
    fn infix_precedence(token: &Self::Token) -> Option<u32>;
    fn partial_parse(tokens: &mut Self::Tokens, precedence: u32) -> Tree<Self> {
        todo!();
    }
    fn parse(src: &'a str) -> Tree<Self> {
        Self::partial_parse(&mut src.into(), 0)
    }
}
