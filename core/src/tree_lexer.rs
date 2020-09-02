pub use crate::lexer::Bracket;
pub use crate::lexer::Keyword;
pub use crate::lexer::Operator;
pub use crate::lexer::Separator;

pub struct FlatTree<'a>(Vec<(&'a str, Node<'a>)>);
enum Node<'a> {
    Token(Token<'a>),
    Tree(Bracket, usize),
}
pub enum Token<'a> {
    Num(&'a str),
    Str(&'a str),
    Char(&'a str),
    Keyword(Keyword),
    Identifier(&'a str),
    Separator(Separator),
    Operator(Operator),
}
pub enum Error<'a> {
    UnterminatedQuote(char, &'a str),
    InvalidToken(&'a str),
    Mismatch((&'a str, Bracket), (&'a str, Bracket)),
    Unexpected(&'a str, Bracket),
    Unmatched(&'a str, Bracket),
}
