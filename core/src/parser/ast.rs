use crate::parser::NodeType;
use util::tree_vec::Tree;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Node<'a> {
    pub span: &'a str,
    pub node: NodeType,
}
pub type Ast<'a> = Tree<Node<'a>>;
pub(super) struct KindedAst<'a> {
    pub ast: Ast<'a>,
    pub kind: AstType,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum AstType {
    Expr,
    Unpack,
    ExprOrUnpack,
}
impl AstType {
    pub fn is_expr(self) -> bool {
        matches!(self, Self::Expr | Self::ExprOrUnpack)
    }
    pub fn is_unpack(self) -> bool {
        matches!(self, Self::Unpack | Self::ExprOrUnpack)
    }
}
