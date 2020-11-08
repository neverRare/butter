#[derive(Clone, Copy)]
pub enum ErrorType {
    NonPlaceAssign,
    NonExpr,
    NonIdent,
    SuddenEof,
    UnknownToken,
    UnterminatedQuote,
}
