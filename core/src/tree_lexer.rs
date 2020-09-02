pub use crate::lexer::Bracket;
pub use crate::lexer::Keyword;
use crate::lexer::Opening;
pub use crate::lexer::Operator;
pub use crate::lexer::Separator;
use crate::lexer::Token as SrcToken;
use util::lexer::Lex;
use util::span::Span;

#[derive(Clone, Copy)]
pub enum Token<'a> {
    Num(&'a str),
    Str(&'a str),
    Char(&'a str),
    Keyword(Keyword),
    Identifier(&'a str),
    Separator(Separator),
    Operator(Operator),
}
impl<'a> Token<'a> {
    fn from_token(token: &SrcToken<'a>) -> Option<Self> {
        Some(match token {
            SrcToken::Num(num) => Self::Num(num),
            SrcToken::Str(content) => Self::Str(content),
            SrcToken::Char(content) => Self::Char(content),
            SrcToken::Keyword(keyword) => Self::Keyword(*keyword),
            SrcToken::Identifier(identifier) => Self::Identifier(identifier),
            SrcToken::Separator(separator) => Self::Separator(*separator),
            SrcToken::Operator(operator) => Self::Operator(*operator),
            _ => return None,
        })
    }
}
#[derive(Clone, Copy)]
pub enum Error<'a> {
    UnterminatedQuote(char, &'a str),
    InvalidToken(&'a str),
    Mismatch((&'a str, Bracket), (&'a str, Bracket)),
    Unexpected(&'a str, Bracket),
    Unmatched(&'a str, Bracket),
}
impl<'a> Error<'a> {
    fn from_token(token: &SrcToken<'a>) -> Option<Self> {
        Some(match token {
            SrcToken::UnterminatedQuote(quote, span) => Self::UnterminatedQuote(*quote, span),
            SrcToken::InvalidToken(span) => Self::InvalidToken(span),
            _ => return None,
        })
    }
}
#[derive(Clone, Copy)]
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
        let mut brackets = vec![];
        let mut stack = vec![];
        let mut current = vec![];
        for (span, token) in SrcToken::lex_span(src) {
            if let SrcToken::Bracket(opening, bracket) = token {
                match opening {
                    Opening::Open => {
                        stack.push(current);
                        brackets.push((span, bracket));
                        current = vec![];
                    }
                    Opening::Close => {
                        if let Some((open_span, open)) = brackets.pop() {
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
                        } else {
                            current.push((span, Node::Error(Error::Unexpected(span, bracket))))
                        }
                    }
                }
            } else if let Some(token) = Token::from_token(&token) {
                current.push((span, Node::Token(token)));
            } else if let Some(error) = Error::from_token(&token) {
                current.push((span, Node::Error(error)));
            } else {
                unreachable!()
            }
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
impl<'a, 'b> Tree<'a, 'b> {
    pub fn iter(self) -> TreeIter<'a, 'b> {
        self.into_iter()
    }
}
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
            self.src = &self.src[1..];
            let token = match node {
                Node::Token(token) => TokenTree::Token(token),
                Node::Tree(bracket, len) => {
                    let tree = Tree(&self.src[..*len]);
                    self.src = &self.src[*len..];
                    TokenTree::Tree(bracket, tree)
                }
                Node::Error(msg) => TokenTree::Error(msg),
            };
            Some((span, token))
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.src.len();
        (1.min(len), Some(len))
    }
}
#[cfg(test)]
mod test {
    use super::BigTree;
    use super::Bracket;
    use super::Token;
    use super::TokenTree;
    #[test]
    fn tree_lex() {
        let big_tree = BigTree::new("(ident){[]}");
        let mut iter = big_tree.tree().into_iter();
        let (span, token) = iter.next().unwrap();
        assert_eq!(span, "(ident)");
        if let TokenTree::Tree(Bracket::Paren, token) = token {
            let mut iter = token.into_iter();
            let (span, token) = iter.next().unwrap();
            assert_eq!(span, "ident");
            assert!(matches!(
                token,
                TokenTree::Token(Token::Identifier("ident"))
            ));
            assert!(iter.next().is_none());
        } else {
            panic!()
        }
        let (span, token) = iter.next().unwrap();
        assert_eq!(span, "{[]}");
        if let TokenTree::Tree(Bracket::Brace, token) = token {
            let mut iter = token.into_iter();
            let (span, token) = iter.next().unwrap();
            assert_eq!(span, "[]");
            if let TokenTree::Tree(Bracket::Bracket, token) = token {
                let mut iter = token.into_iter();
                assert!(iter.next().is_none());
            } else {
                panic!()
            }
            assert!(iter.next().is_none());
        } else {
            panic!()
        }
        assert!(iter.next().is_none());
    }
}
