pub use crate::lexer::Bracket;
pub use crate::lexer::Keyword;
use crate::lexer::Opening;
pub use crate::lexer::Operator;
pub use crate::lexer::Separator;
use crate::lexer::Token as SrcToken;
use util::lexer::Lex;

pub struct BigTree<'a>(Vec<(&'a str, Node<'a>)>);
impl<'a> BigTree<'a> {
    fn new(src: &'a str) -> Self {
        let mut brackets: Vec<(&str, Bracket)> = vec![];
        let mut stack: Vec<Vec<(&str, Node)>> = vec![];
        let mut current: Vec<(&str, Node)> = vec![];
        for (span, token) in SrcToken::lex_span(src) {
            let token = match token {
                SrcToken::Whitespace => continue,
                SrcToken::Comment(_) => continue,
                SrcToken::Bracket(Opening::Open, bracket) => {
                    stack.push(current);
                    brackets.push((span, bracket));
                    current = vec![];
                    continue;
                }
                SrcToken::Bracket(Opening::Close, bracket) => todo!(),
                SrcToken::Num(num) => Token::Num(num),
                SrcToken::Str(content) => Token::Str(content),
                SrcToken::Char(content) => Token::Char(content),
                SrcToken::Keyword(keyword) => Token::Keyword(keyword),
                SrcToken::Identifier(identifier) => Token::Identifier(identifier),
                SrcToken::Separator(separator) => Token::Separator(separator),
                SrcToken::Operator(separator) => Token::Operator(separator),
                SrcToken::UnterminatedQuote(quote, src) => {
                    current.push((span, Node::Error(Error::UnterminatedQuote(quote, src))));
                    continue;
                }
                SrcToken::InvalidToken(src) => {
                    current.push((span, Node::Error(Error::InvalidToken(src))));
                    continue;
                }
            };
            current.push((span, Node::Token(token)));
        }
        Self(current)
    }
}
enum Node<'a> {
    Token(Token<'a>),
    Tree(Bracket, usize),
    Error(Error<'a>),
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
