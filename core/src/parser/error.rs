#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErrorType {
    NonPlace,
    NonExpr,
    NoIdent,
    NoOptionalChain,
    NoExpr,
    NoBlock,
    UnknownToken,
    UnterminatedQuote,
    IntegerOverflow,
    ExpOverflow,
    InvalidNumber,
    InvalidEscape,
    NonSingleChar,
}
