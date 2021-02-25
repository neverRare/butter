#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErrorType {
    NonPlace,
    NoIdent,
    NoOptionalChain,
    NoExpr,
    NoBlock,
    NoIfNorBlock,
    UnterminatedQuote,
    IntegerOverflow,
    ExpOverflow,
    InvalidNumber,
    InvalidEscape,
    NonSingleChar,
}
