use crate::lexer::Keyword;
use crate::lexer::Operator;
use crate::lexer::Token;
use crate::parser::Node;
use crate::parser::NodeType;
use crate::parser::SpanToken;
use crate::parser::UnaryOp;
use std::iter::Peekable;
use util::parser::Parser;
use util::tree_vec::Tree;

pub(super) fn keyword_literal<'a>(
    prefix: SpanToken<'a>,
    _: &mut Peekable<impl Iterator<Item = SpanToken<'a>>>,
) -> Option<Tree<Node<'a>>> {
    let SpanToken {
        span,
        token: prefix,
    } = prefix;
    if let Token::Keyword(keyword) = prefix {
        let node = match keyword {
            Keyword::Abort => NodeType::Abort,
            Keyword::True => NodeType::True,
            Keyword::False => NodeType::False,
            Keyword::Null => NodeType::Null,
            _ => return None,
        };
        Some(Tree::new(Node {
            span,
            node,
            unpack: false,
        }))
    } else {
        None
    }
}
pub(super) fn clone<'a>(
    prefix: SpanToken<'a>,
    tokens: &mut Peekable<impl Iterator<Item = SpanToken<'a>>>,
) -> Option<Tree<Node<'a>>> {
    let SpanToken {
        span,
        token: prefix,
    } = prefix;
    if let Token::Keyword(Keyword::Clone) = prefix {
        let operand = Node::partial_parse(tokens, 90);
        Some(Tree {
            content: Node {
                span: span.up_to(operand.content.span),
                node: NodeType::Unary(UnaryOp::Clone),
                unpack: false,
            },
            children: operand.into_tree_vec(),
        })
    } else {
        None
    }
}
pub(super) fn operator<'a>(
    prefix: SpanToken<'a>,
    tokens: &mut Peekable<impl Iterator<Item = SpanToken<'a>>>,
) -> Option<Tree<Node<'a>>> {
    let SpanToken {
        span,
        token: prefix,
    } = prefix;
    if let Token::Operator(operator) = prefix {
        let operator = match operator {
            Operator::Plus => UnaryOp::Plus,
            Operator::Minus => UnaryOp::Minus,
            Operator::Bang => UnaryOp::Not,
            Operator::Amp => UnaryOp::Ref,
            _ => return None,
        };
        let operand = Node::partial_parse(tokens, 90);
        Some(Tree {
            content: Node {
                span: span.up_to(operand.content.span),
                node: NodeType::Unary(operator),
                unpack: false,
            },
            children: operand.into_tree_vec(),
        })
    } else {
        None
    }
}
