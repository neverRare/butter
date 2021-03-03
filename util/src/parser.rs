use crate::iter::PeekableIterator;

pub trait ParserIter: PeekableIterator {
    type Ast;
    type Kind;
    fn prefix_parse(&mut self, kind: &Self::Kind) -> Self::Ast;
    fn infix_parse(
        &mut self,
        left_node: Self::Ast,
        infix: Self::Item,
        kind: &Self::Kind,
    ) -> Self::Ast;
    fn infix_precedence(token: &Self::Item, kind: &Self::Kind) -> Option<u32>;
    fn partial_parse(&mut self, precedence: u32, kind: &Self::Kind) -> Self::Ast {
        let mut node = Self::prefix_parse(self, kind);
        while let Some(token) = self.peek() {
            if Self::infix_precedence(token, kind)
                .map(|num| num <= precedence)
                .unwrap_or(true)
            {
                break;
            }
            let infix = self.next().unwrap();
            node = self.infix_parse(node, infix, kind);
        }
        node
    }
}
#[cfg(test)]
mod test {
    use crate::iter::PeekableIterator;
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
    impl<T: PeekableIterator> PeekableIterator for Parser<T> {
        fn peek(&mut self) -> Option<&Self::Item> {
            T::peek(&mut self.0)
        }
    }
    impl<T: PeekableIterator<Item = Token>> ParserIter for Parser<T> {
        type Ast = Option<Tree<Node>>;
        type Kind = ();
        fn prefix_parse(&mut self, kind: &Self::Kind) -> Self::Ast {
            match self.peek()? {
                Token::Num => {
                    self.next();
                    Some(Tree::new(Node::Num))
                }
                Token::OpenGroup => {
                    self.next();
                    let inside = self.partial_parse(0, &());
                    if let Token::CloseGroup = self.peek()? {
                        inside
                    } else {
                        None
                    }
                }
                Token::Prefix => {
                    self.next();
                    Some(Tree {
                        content: Node::Prefix,
                        children: crate::join_trees![self.partial_parse(30, &())?],
                    })
                }
                _ => None,
            }
        }
        fn infix_parse(
            &mut self,
            left_node: Self::Ast,
            infix: Self::Item,
            kind: &Self::Kind,
        ) -> Self::Ast {
            let (content, precedence) = match infix {
                Token::InfixLeft => (Node::InfixLeft, 10),
                Token::InfixRight => (Node::InfixRight, 19),
                _ => unreachable!(),
            };
            let right = self.partial_parse(precedence, &())?;
            Some(Tree {
                content,
                children: crate::join_trees![left_node?, right],
            })
        }
        fn infix_precedence(infix: &Self::Item, kind: &Self::Kind) -> Option<u32> {
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
                Parser([$($token),*].iter().copied().peekable()).partial_parse(0, &()),
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
