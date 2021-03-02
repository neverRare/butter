use crate::lexer::Bracket;
use crate::lexer::Keyword;
use crate::lexer::Opening;
use crate::lexer::Operator;
use crate::lexer::Separator;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErrorType {
    NonPlace,
    NoExpectation(&'static [TokenKind]),
    NoExpr,
    NonIndexNorSlice,
    UnterminatedQuote,
    IntegerOverflow,
    ExpOverflow,
    InvalidNumber,
    InvalidEscape,
    NonSingleChar,
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
