use crate::lexer::Token;
use crate::parser::Node;
use util::tree_vec::Tree;
use util::tree_vec::TreeVec;

pub enum StrError {
    InvalidEscape,
    InvalidHex,
    TerminatedEscape,
}
type StrErrorSpans<'a> = Vec<(&'a str, StrError)>;
fn parse_inner_string(content: &str) -> Result<TreeVec<(&str, Node)>, StrErrorSpans> {
    todo!()
}
pub(super) fn parse_string<'a>(
    span: &'a str,
    token: Token<'a>,
) -> Result<Tree<(&'a str, Node)>, StrErrorSpans<'a>> {
    let (node, content) = match token {
        Token::Str(content) => (Node::Str, content),
        Token::Char(content) => (Node::Char, content),
        token => panic!("expecting Str or Char, found {:?}", token),
    };
    let children = parse_inner_string(content)?;
    Ok(Tree {
        content: (span, node),
        children,
    })
}
