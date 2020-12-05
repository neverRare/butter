#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErrorType {
    NonPlace,
    NonExpr,
    NoIdent,
    NoExpr,
    UnknownToken,
    UnterminatedQuote,
    IntegerOverflow,
    InvalidNumber,
}
