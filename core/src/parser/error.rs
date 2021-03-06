use crate::lexer::Bracket;
use crate::lexer::Keyword;
use crate::lexer::Opening;
use crate::lexer::Operator;
use crate::lexer::Separator;
use crate::parser::AstType;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErrorType {
    NonPlace,
    NoExpectation(&'static [ExpectedToken]),
    NoExpr,
    NoUnpack,
    NoExprNorUnpack,
    NotNamed,
    RestAfterRest,
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
            AstType::Either => Self::NoExprNorUnpack,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Error<'a> {
    pub span: &'a str,
    pub error: ErrorType,
}
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ExpectedToken {
    Keyword(Keyword),
    Ident,
    Separator(Separator),
    Bracket(Opening, Bracket),
    Operator(Operator),
    // Int,
    // Float,
    // Str,
    // Char,
    // Underscore,
}
