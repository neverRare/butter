use crate::lexer::Bracket;
use crate::lexer::Keyword;
use crate::lexer::Opening;
use crate::lexer::Operator;
use crate::lexer::Separator;
use crate::parser::AstType;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErrorType {
    NonPlace,
    NoExpectation(&'static [TokenKind]),
    NoExpr,
    NoUnpack,
    NoExprNorUnpack,
    RestAfterRest,
    NonIndexNorSlice,
    UnterminatedQuote,
    IntegerOverflow,
    ExpOverflow,
    InvalidNumber,
    InvalidEscape,
    NonSingleChar,
}
impl ErrorType {
    pub(super) fn no_expected_kind(kind: AstType) -> Self {
        match kind {
            AstType::Expr => Self::NoExpr,
            AstType::Unpack => Self::NoUnpack,
            AstType::ExprOrUnpack => Self::NoExprNorUnpack,
        }
    }
}
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TokenKind {
    Int,
    Float,
    Str,
    Char,
    Keyword(Keyword),
    Underscore,
    Ident,
    Separator(Separator),
    Bracket(Opening, Bracket),
    Operator(Operator),
}
