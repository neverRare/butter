use crate::parser::Node;
use crate::parser::Parser;
use util::tree_vec::Tree;
use util::tree_vec::TreeVec;

enum BracketSyntax<'a> {
    Empty,
    Single(Tree<Node<'a>>),
    Multiple(TreeVec<Node<'a>>),
    Range(Tree<Node<'a>>),
}
impl<'a> BracketSyntax<'a> {
    fn parse_rest(parser: &mut Parser<'a>, left_bracket_span: &'a str) -> Self {
        todo!()
    }
}
