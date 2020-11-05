use std::iter::Peekable;

pub struct Parser<I: Iterator>(Peekable<I>);
impl<I: Iterator> Parser<I> {
    pub fn new(iter: I) -> Self {
        Self(iter.peekable())
    }
    pub fn peek(&mut self) -> Option<&I::Item> {
        self.0.peek()
    }
    pub fn partial_parse<T>(&mut self, precedence: u32) -> T::Node
    where
        T: Parse,
        I: Iterator<Item = T>,
    {
        let mut node = Self::prefix_parse(self);
        while let Some(token) = self.peek() {
            if token
                .infix_precedence()
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
    pub fn prefix_parse<T>(&mut self) -> T::Node
    where
        T: Parse,
        I: Iterator<Item = T>,
    {
        T::prefix_parse(self)
    }
    pub fn infix_parse<T>(&mut self, left_node: T::Node, infix: T) -> T::Node
    where
        T: Parse,
        I: Iterator<Item = T>,
    {
        infix.infix_parse(left_node, self)
    }
}
impl<T: Iterator> Iterator for Parser<T> {
    type Item = T::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
pub trait Parse: Sized {
    type Node;
    fn prefix_parse(tokens: &mut Parser<impl Iterator<Item = Self>>) -> Self::Node;
    fn infix_parse(
        &self,
        left_node: Self::Node,
        tokens: &mut Parser<impl Iterator<Item = Self>>,
    ) -> Self::Node;
    fn infix_precedence(&self) -> Option<u32>;
}
#[cfg(test)]
mod test {
    use crate::parser::Parse;
    use crate::parser::Parser;
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
    impl Parse for Token {
        type Node = Option<Tree<Node>>;

        fn prefix_parse(tokens: &mut Parser<impl Iterator<Item = Self>>) -> Self::Node {
            match tokens.next()? {
                Token::Num => Some(Tree::new(Node::Num)),
                Token::OpenGroup => {
                    let inside = tokens.partial_parse(0);
                    if let Token::CloseGroup = tokens.peek()? {
                        inside
                    } else {
                        None
                    }
                }
                Token::Prefix => Some(Tree {
                    content: Node::Prefix,
                    children: tokens.partial_parse(30)?.into_tree_vec(),
                }),
                _ => None,
            }
        }
        fn infix_parse(
            &self,
            left_node: Self::Node,
            tokens: &mut Parser<impl Iterator<Item = Self>>,
        ) -> Self::Node {
            let (content, precedence) = match self {
                Token::InfixLeft => (Node::InfixLeft, 10),
                Token::InfixRight => (Node::InfixRight, 19),
                _ => unreachable!(),
            };
            let mut children = left_node?.into_tree_vec();
            children.push(tokens.partial_parse(precedence)?);
            Some(Tree { content, children })
        }
        fn infix_precedence(&self) -> Option<u32> {
            Some(match self {
                Token::InfixLeft => 10,
                Token::InfixRight => 20,
                _ => return None,
            })
        }
    }
    macro_rules! assert_parser {
        ([$($token:expr),* $(,)?], $content:expr => {$($children:tt)*} $(,)?) => {
            assert_eq!(
                Parser::new([$($token),*].iter().copied()).partial_parse(0),
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
