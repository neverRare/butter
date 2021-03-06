use crate::lexer::Bracket;
use crate::lexer::Opening;
use crate::lexer::Token;
use crate::parser::error_start;
use crate::parser::AstResult;
use crate::parser::ErrorType;
use crate::parser::ExpectedToken;
use crate::parser::Parser;
use util::iter::PeekableIterator;

pub(super) fn block_rest<'a>(parser: &mut Parser<'a>, left_bracket_span: &'a str) -> AstResult<'a> {
    todo!()
}
pub(super) fn block<'a>(parser: &mut Parser<'a>) -> AstResult<'a> {
    let err_span = if let Some(token) = parser.peek() {
        if token.token != Token::Bracket(Opening::Open, Bracket::Brace) {
            let span = token.span;
            Some(&span[..0])
        } else {
            None
        }
    } else {
        let src = parser.src;
        Some(&src[src.len()..])
    };
    if let Some(span) = err_span {
        return Err(error_start(
            span,
            ErrorType::NoExpectation(&[ExpectedToken::Bracket(Opening::Open, Bracket::Brace)]),
        ));
    }
    let bracket = parser.next().unwrap();
    block_rest(parser, bracket.span)
}
pub(super) struct Statements<'a, 'b> {
    parser: &'b mut Parser<'a>,
    pub(super) semicolon: bool,
    resumable: bool,
}
impl<'a, 'b> Iterator for Statements<'a, 'b> {
    type Item = AstResult<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
