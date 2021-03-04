use crate::parser::ast::AstVec;
use crate::parser::Ast;
use crate::parser::AstType;
use crate::parser::Parser;
use crate::parser::ParserResult;

pub(super) enum ParenthesisSyntax<'a> {
    Empty,
    SingleIdent(&'a str),
    SingleExpr(Ast<'a>),
    Fields(AstVec<'a>),
}
pub(super) struct ParenthesisFragment<'a> {
    pub(super) syntax: ParenthesisSyntax<'a>,
    pub kind: AstType,
    pub right_parenthesis_span: &'a str,
}
impl<'a> ParenthesisFragment<'a> {
    pub fn parse_rest(parser: &mut Parser<'a>, kind: AstType, arg: bool) -> ParserResult<'a, Self> {
        todo!()
    }
}
