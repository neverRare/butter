#[derive(Clone, Copy)]
pub enum ErrorType {
    NonPlaceAssign,
    NonExpr,
    SuddenEof,
    UnknownToken,
    UnterminatedQuote,
}
