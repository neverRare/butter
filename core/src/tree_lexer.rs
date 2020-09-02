pub use crate::lexer::Bracket;
pub use crate::lexer::Keyword;
use crate::lexer::Opening;
pub use crate::lexer::Operator;
pub use crate::lexer::Separator;
use crate::lexer::Token as SrcToken;
use util::lexer::Lex;
use util::span::Span;

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
pub enum TokenTree<'a, 'b> {
    Token(&'b Token<'a>),
    Tree(&'b Bracket, Tree<'a, 'b>),
    Error(&'b Error<'a>),
}
enum Node<'a> {
    Token(Token<'a>),
    Tree(Bracket, usize),
    Error(Error<'a>),
}
pub struct BigTree<'a>(Vec<(&'a str, Node<'a>)>);
impl<'a> BigTree<'a> {
    pub fn new(src: &'a str) -> Self {
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
                SrcToken::Bracket(Opening::Close, bracket) => {
                    match brackets.pop() {
                        Some((open_span, open)) => {
                            let big_span = Span::from_spans(src, &open_span, &span);
                            let mut prev_current = current;
                            let mut next_current = stack.pop().unwrap();
                            if open == bracket {
                                next_current.push((big_span, Node::Tree(open, prev_current.len())));
                                next_current.append(&mut prev_current);
                            } else {
                                next_current.push((
                                    big_span,
                                    Node::Error(Error::Mismatch(
                                        (open_span, open),
                                        (span, bracket),
                                    )),
                                ))
                            }
                            current = next_current;
                        }
                        None => current.push((span, Node::Error(Error::Unexpected(span, bracket)))),
                    }
                    continue;
                }
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
        while let Some((span, bracket)) = brackets.pop() {
            current.push((span, Node::Error(Error::Unmatched(span, bracket))))
        }
        Self(current)
    }
    pub fn tree<'b>(&'b self) -> Tree<'a, 'b> {
        self.into()
    }
}
#[derive(Clone, Copy)]
pub struct Tree<'a, 'b>(&'b [(&'a str, Node<'a>)]);
impl<'a, 'b> From<&'b BigTree<'a>> for Tree<'a, 'b> {
    fn from(BigTree(vec): &'b BigTree<'a>) -> Self {
        Self(vec)
    }
}
impl<'a, 'b> IntoIterator for Tree<'a, 'b> {
    type Item = (&'a str, TokenTree<'a, 'b>);
    type IntoIter = TreeIter<'a, 'b>;
    fn into_iter(self) -> Self::IntoIter {
        TreeIter { src: self.0 }
    }
}
pub struct TreeIter<'a, 'b> {
    src: &'b [(&'a str, Node<'a>)],
}
impl<'a, 'b> Iterator for TreeIter<'a, 'b> {
    type Item = (&'a str, TokenTree<'a, 'b>);
    fn next(&mut self) -> Option<Self::Item> {
        if self.src.is_empty() {
            None
        } else {
            let (span, node) = &self.src[0];
            let token = match node {
                Node::Token(token) => {
                    self.src = &self.src[1..];
                    TokenTree::Token(token)
                }
                Node::Tree(bracket, len) => {
                    let tree = Tree(&self.src[1 + len..]);
                    self.src = &self.src[1..];
                    TokenTree::Tree(bracket, tree)
                }
                Node::Error(msg) => {
                    self.src = &self.src[1..];
                    TokenTree::Error(msg)
                }
            };
            Some((span, token))
        }
    }
}
