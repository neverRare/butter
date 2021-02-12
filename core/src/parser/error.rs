#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErrorType {
    NonPlace,
    NonExpr,
    NoIdent,
    NoOptionalChain,
    NoExpr,
    UnknownToken,
    UnterminatedQuote,
    IntegerOverflow,
    InvalidNumber,
    InvalidEscape,
    NonSingleChar,
}
