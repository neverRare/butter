#[derive(Clone, Copy)]
pub enum ErrorType {
    NonPlaceAssign,
    NonExprOperand,
    NoIdentAfterDot,
    SuddenEof,
    UnknownToken,
    UnterminatedQuote,
}
