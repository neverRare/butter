use crate::tree_vec::Tree;
use std::iter::Peekable;

// TODO: better error handling
// TODO: test
pub trait Parser: Sized {
    type Token;
    fn error_node() -> Self;
    fn prefix_parse(
        prefix: Self::Token,
        tokens: &mut Peekable<impl Iterator<Item = Self::Token>>,
    ) -> Tree<Self>;
    fn infix_parse(
        left_node: Tree<Self>,
        infix: Self::Token,
        tokens: &mut Peekable<impl Iterator<Item = Self::Token>>,
    ) -> Tree<Self>;
    fn infix_precedence(token: &Self::Token) -> Option<u32>;
    fn partial_parse(
        tokens: &mut Peekable<impl Iterator<Item = Self::Token>>,
        precedence: u32,
    ) -> Tree<Self> {
        let token = match tokens.next() {
            Some(node) => node,
            None => return Tree::new(Self::error_node()),
        };
        let mut node = Self::prefix_parse(token, tokens);
        while let Some(token) = tokens.next() {
            if Self::infix_precedence(&token)
                .map(|num| num < precedence)
                .unwrap_or(true)
            {
                break;
            }
            node = Self::infix_parse(node, token, tokens);
        }
        node
    }
}
