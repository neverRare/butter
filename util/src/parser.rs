use crate::iter::PeekableIter;

pub trait ParserIter: PeekableIter {
    type Node;
    fn prefix_parse(&mut self) -> Self::Node;
    fn infix_parse(&mut self, left_node: Self::Node, infix: Self::Item) -> Self::Node;
    fn infix_precedence(token: &Self::Item) -> Option<u32>;
    fn partial_parse(&mut self, precedence: u32) -> Self::Node {
        let mut node = Self::prefix_parse(self);
        while let Some(token) = self.peek() {
            if Self::infix_precedence(token)
                .map(|num| num <= precedence)
                .unwrap_or(true)
            {
                break;
            }
            let infix = self.next().unwrap();
            node = self.infix_parse(node, infix);
        }
        node
    }
}
#[cfg(test)]
mod test {
    use crate::iter::PeekableIter;
    use crate::parser::ParserIter;
    use crate::tree_vec;
    use crate::tree_vec::Tree;

    #[derive(PartialEq, Eq, Clone, Copy, Debug)]
    enum Node {
        Num,
        Prefix,
        InfixLeft,
        InfixRight,
    }
    #[derive(PartialEq, Eq, Clone, Copy, Debug)]
    enum Token {
        Num,
        OpenGroup,
        CloseGroup,
        Prefix,
        InfixLeft,
        InfixRight,
    }
    struct Parser<T>(T);
    impl<T: Iterator> Iterator for Parser<T> {
        type Item = T::Item;
        fn next(&mut self) -> Option<Self::Item> {
            T::next(&mut self.0)
        }
    }
    impl<T: PeekableIter> PeekableIter for Parser<T> {
        fn peek(&mut self) -> Option<&Self::Item> {
            T::peek(&mut self.0)
        }
    }
    impl<T: PeekableIter<Item = Token>> ParserIter for Parser<T> {
        type Node = Option<Tree<Node>>;
        fn prefix_parse(&mut self) -> Self::Node {
            match self.next()? {
                Token::Num => Some(Tree::new(Node::Num)),
                Token::OpenGroup => {
                    let inside = self.partial_parse(0);
                    if let Token::CloseGroup = self.peek()? {
                        inside
                    } else {
                        None
                    }
                }
                Token::Prefix => Some(Tree {
                    content: Node::Prefix,
                    children: crate::join_trees![self.partial_parse(30)?],
                }),
                _ => None,
            }
        }
        fn infix_parse(&mut self, left_node: Self::Node, infix: Self::Item) -> Self::Node {
            let (content, precedence) = match infix {
                Token::InfixLeft => (Node::InfixLeft, 10),
                Token::InfixRight => (Node::InfixRight, 19),
                _ => unreachable!(),
            };
            Some(Tree {
                content,
                children: crate::join_trees![left_node?, self.partial_parse(precedence)?],
            })
        }
        fn infix_precedence(infix: &Self::Item) -> Option<u32> {
            Some(match infix {
                Token::InfixLeft => 10,
                Token::InfixRight => 20,
                _ => return None,
            })
        }
    }
    macro_rules! assert_parser {
        ([$($token:expr),* $(,)?], $content:expr => {$($children:tt)*} $(,)?) => {
            assert_eq!(
                Parser([$($token),*].iter().copied().peekable()).partial_parse(0),
                Some(Tree {
                    content: $content,
                    children: tree_vec! { $($children)* },
                }),
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
