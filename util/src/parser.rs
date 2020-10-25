use crate::tree_vec::Tree;

trait Parser: Sized {
    type Tokens: Iterator;
    type Error;
    fn error_node() -> Self;
    fn simple_node(tokens: &mut Self::Tokens) -> Result<Tree<Self>, Self::Error>;
    fn partial_node(tokens: &mut Self::Tokens) -> Result<Tree<Self>, Self::Error>;
    fn precedence(tree: &Self) -> u32;
}
