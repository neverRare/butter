#[derive(Clone, Copy)]
pub enum ErrorType {
    NonPlaceAssign,
    NonExprOperand,
    NoExpr,
    UnknownToken,
    UnterminatedQuote,
}
