use crate::parser::NodeType;
use util::tree_vec::Tree;
use util::tree_vec::TreeVec;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Node<'a> {
    pub span: &'a str,
    pub node: NodeType,
}
pub type Ast<'a> = Tree<Node<'a>>;
pub type AstVec<'a> = TreeVec<Node<'a>>;
pub(super) struct KindedAst<'a> {
    pub ast: Ast<'a>,
    pub kind: AstType,
}
impl<'a> KindedAst<'a> {
    pub fn new_expr(ast: Ast<'a>) -> Self {
        Self {
            ast,
            kind: AstType::Expr,
        }
    }
    pub fn new_unpack(ast: Ast<'a>) -> Self {
        Self {
            ast,
            kind: AstType::Unpack,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum AstType {
    Expr,
    Unpack,
    Either,
}
impl AstType {
    pub fn is_expr(self) -> bool {
        matches!(self, Self::Expr | Self::Either)
    }
    pub fn is_unpack(self) -> bool {
        matches!(self, Self::Unpack | Self::Either)
    }
}
