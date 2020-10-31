use crate::tree_vec::Tree;
use std::iter::Peekable;

// TODO: better error handling
pub trait Parser: Sized {
    type Token;
    fn prefix_parse(tokens: &mut Peekable<impl Iterator<Item = Self::Token>>) -> Tree<Self>;
    fn infix_parse(
        left_node: Tree<Self>,
        infix: Self::Token,
        tokens: &mut Peekable<impl Iterator<Item = Self::Token>>,
    ) -> Tree<Self>;
    fn infix_precedence(token: &Self::Token) -> Option<u32>;
    fn partial_parse(
        tokens: &mut Peekable<impl Iterator<Item = Self::Token>>,
        precedence: u32,
    ) -> Tree<Self> {
        let mut node = Self::prefix_parse(tokens);
        while let Some(token) = tokens.peek() {
            if Self::infix_precedence(token)
                .map(|num| num <= precedence)
                .unwrap_or(true)
            {
                break;
            }
            node = Self::infix_parse(node, tokens.next().unwrap(), tokens);
        }
        node
    }
}
#[cfg(test)]
mod test {
    use crate::parser::Parser;
    use crate::tree_vec;
    use crate::tree_vec::Tree;
    use std::iter::Peekable;

    #[derive(PartialEq, Eq, Clone, Copy, Debug)]
    enum Token {
        Num,
        OpenGroup,
        CloseGroup,
        Prefix,
        InfixLeft,
        InfixRight,
    }
    #[derive(PartialEq, Eq, Clone, Copy, Debug)]
    enum Node {
        Num,
        Prefix,
        InfixLeft,
        InfixRight,
        Error,
    }
    impl Parser for Node {
        type Token = Token;
        fn prefix_parse(tokens: &mut Peekable<impl Iterator<Item = Self::Token>>) -> Tree<Self> {
            let prefix = match tokens.next() {
                Some(token) => token,
                None => return Tree::new(Self::Error),
            };
            match prefix {
                Token::Num => Tree::new(Self::Num),
                Token::OpenGroup => {
                    let inside = Self::partial_parse(tokens, 0);
                    if let Some(Token::CloseGroup) = tokens.peek() {
                        inside
                    } else {
                        Tree::new(Self::Error)
                    }
                }
                Token::Prefix => Tree {
                    content: Self::Prefix,
                    children: Self::partial_parse(tokens, 30).into_tree_vec(),
                },
                _ => Tree::new(Self::Error),
            }
        }
        fn infix_parse(
            left_node: Tree<Self>,
            infix: Self::Token,
            tokens: &mut Peekable<impl Iterator<Item = Self::Token>>,
        ) -> Tree<Self> {
            let (content, precedence) = match infix {
                Token::InfixLeft => (Self::InfixLeft, 10),
                Token::InfixRight => (Self::InfixRight, 19),
                _ => unreachable!(),
            };
            let mut children = left_node.into_tree_vec();
            children.push(Self::partial_parse(tokens, precedence));
            Tree { content, children }
        }
        fn infix_precedence(token: &Self::Token) -> Option<u32> {
            Some(match token {
                Token::InfixLeft => 10,
                Token::InfixRight => 20,
                _ => return None,
            })
        }
    }
    macro_rules! assert_parser {
        ([$($token:expr),* $(,)?], $content:expr => {$($children:tt)*} $(,)?) => {
            assert_eq!(
            Node::partial_parse(&mut [$($token),*].iter().copied().peekable(), 0),
            Tree {
                content: $content,
                children: tree_vec! { $($children)* },
            },
        );
        };
    }
    #[test]
    fn infix() {
        assert_parser!(
            [
                Token::Num,
                Token::InfixLeft,
                Token::Num,
                Token::InfixRight,
                Token::Num,
            ],
            Node::InfixLeft => {
                Node::Num,
                Node::InfixRight => {
                    Node::Num,
                    Node::Num,
                },
            },
        );
        assert_parser!(
            [
                Token::Num,
                Token::InfixRight,
                Token::Num,
                Token::InfixLeft,
                Token::Num,
            ],
            Node::InfixLeft => {
                Node::InfixRight => {
                    Node::Num,
                    Node::Num,
                },
                Node::Num,
            },
        );
    }
    #[test]
    fn associativity() {
        assert_parser!(
            [
                Token::Num,
                Token::InfixLeft,
                Token::Num,
                Token::InfixLeft,
                Token::Num,
            ],
            Node::InfixLeft => {
                Node::InfixLeft => {
                    Node::Num,
                    Node::Num,
                },
                Node::Num,
            },
        );
        assert_parser!(
            [
                Token::Num,
                Token::InfixRight,
                Token::Num,
                Token::InfixRight,
                Token::Num,
            ],
            Node::InfixRight => {
                Node::Num,
                Node::InfixRight => {
                    Node::Num,
                    Node::Num,
                },
            },
        );
    }
    #[test]
    fn prefix() {
        assert_parser!(
            [
                Token::Prefix,
                Token::Prefix,
                Token::Num,
            ],
            Node::Prefix => {
                Node::Prefix => {
                    Node::Num,
                },
            },
        );
        assert_parser!(
            [
                Token::Prefix,
                Token::Num,
                Token::InfixLeft,
                Token::Num,
            ],
            Node::InfixLeft => {
                Node::Prefix => {
                    Node::Num,
                },
                Node::Num,
            },
        );
        assert_parser!(
            [
                Token::Num,
                Token::InfixLeft,
                Token::Prefix,
                Token::Num,
            ],
            Node::InfixLeft => {
                Node::Num,
                Node::Prefix => {
                    Node::Num,
                },
            },
        );
    }
    #[test]
    fn group() {
        assert_parser!(
            [
                Token::Prefix,
                Token::OpenGroup,
                Token::Num,
                Token::InfixLeft,
                Token::Num,
                Token::CloseGroup,
            ],
            Node::Prefix => {
                Node::InfixLeft => {
                    Node::Num,
                    Node::Num,
                },
            },
        );
    }
}
