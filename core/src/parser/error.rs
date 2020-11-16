use crate::parser::number::Radix;

#[derive(Clone, Copy)]
pub enum ErrorType {
    NonPlace,
    NonExpr,
    NoIdent,
    NoExpr,
    UnknownToken,
    UnterminatedQuote,
    InvalidDigit(Radix),
    DoubleDecimal,
    DoubleMantissa,
    DecimalOnMantissa,
    DecimalOnInteger,
    IntegerOverflow,
}
