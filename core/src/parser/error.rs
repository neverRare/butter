#[derive(Clone, Copy)]
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
