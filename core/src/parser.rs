use crate::lexer::Bracket;
use crate::lexer::Opening;
use crate::lexer::Operator;
use crate::lexer::Token;
use crate::parser::node_type::NodeType;
use util::mini_fn;
use util::parser::Parse;
use util::parser::Parser;
use util::span::Span;
use util::tree_vec::Tree;

mod infix_parselet;
mod node_type;
mod prefix_parselet;

#[derive(Clone, Copy)]
struct Node<'a> {
    span: Span<'a>,
    node: NodeType,
    unpack: bool,
}
#[derive(Clone, Copy)]
struct SpanToken<'a> {
    span: Span<'a>,
    token: Token<'a>,
}
impl<'a> Parse for SpanToken<'a> {
    type Node = Tree<Node<'a>>;
    fn prefix_parse(tokens: &mut Parser<impl Iterator<Item = Self>>) -> Self::Node {
        let prefix = tokens.next().unwrap();
        mini_fn! {
            (prefix, tokens);
            prefix_parselet::operator,
            prefix_parselet::clone,
            prefix_parselet::keyword_literal,
            => else panic!("Prefix token remained unhandled: {:?}", prefix.token),
        }
    }
    fn infix_parse(
        left_node: Self::Node,
        infix: Self,
        tokens: &mut Parser<impl Iterator<Item = Self>>,
    ) -> Self::Node {
        mini_fn! {
            (left_node, infix, tokens);
            infix_parselet::operator,
            => else panic!("Prefix token remained unhandled: {:?}", infix.token),
        }
    }
    fn infix_precedence(&self) -> Option<u32> {
        Some(match self.token {
            Token::Bracket(Opening::Open, Bracket::Bracket) => 100,
            Token::Bracket(Opening::Open, Bracket::Paren) => 100,
            Token::Operator(operator) => match operator {
                Operator::Dot => 100,
                Operator::Star => 80,
                Operator::Slash => 80,
                Operator::DoubleSlash => 80,
                Operator::Percent => 80,
                Operator::Plus => 70,
                Operator::Minus => 70,
                Operator::PlusPlus => 70,
                Operator::DoubleEqual => 60,
                Operator::NotEqual => 60,
                Operator::Less => 60,
                Operator::LessEqual => 60,
                Operator::Greater => 60,
                Operator::GreaterEqual => 60,
                Operator::Amp => 50,
                Operator::DoubleAmp => 50,
                Operator::Pipe => 40,
                Operator::DoublePipe => 40,
                Operator::DoubleQuestion => 30,
                Operator::LeftArrow => 20,
                _ => return None,
            },
            _ => return None,
        })
    }
}
