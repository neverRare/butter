use crate::parser::SpanToken;
use crate::parser::Token;
use util::lexer::LexFilter;
use util::lexer::SpanFilterIter;
use util::span::Span;

pub(super) struct RawLexer<'a> {
    src: &'a str,
    iter: SpanFilterIter<'a, Token<'a>>,
}
impl<'a> RawLexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            iter: Token::lex_span(src),
        }
    }
}
impl<'a> Iterator for RawLexer<'a> {
    type Item = SpanToken<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let (span, token) = self.iter.next()?;
        Some(SpanToken {
            span: Span::from_str(self.src, span),
            token,
        })
    }
}
