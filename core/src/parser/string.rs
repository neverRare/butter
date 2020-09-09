use crate::lexer::Token;
use crate::parser::NodeKind;
use util::tree::Tree;
use util::tree::TreeVec;

pub enum StrError {
    InvalidEscape,
    InvalidHex,
    TerminatedEscape,
}
type StrErrorSpans<'a> = Vec<(&'a str, StrError)>;
fn parse_inner_string(content: &str) -> Result<TreeVec<(&str, NodeKind)>, StrErrorSpans> {
    todo!()
}
pub(super) fn parse_string<'a>(
    span: &'a str,
    token: Token<'a>,
) -> Result<Tree<(&'a str, NodeKind)>, StrErrorSpans<'a>> {
    let (node, content) = match token {
        Token::Str(content) => (NodeKind::Str, content),
        Token::Char(content) => (NodeKind::Char, content),
        token => panic!("expecting Str or Char, found {:?}", token),
    };
    let children = parse_inner_string(content)?;
    Ok(Tree {
        content: (span, node),
        children,
    })
}
