use crate::lexer::Operator;
use crate::lexer::Token;
use crate::parser::node_type::BinaryOp;
use crate::parser::node_type::NodeType;
use crate::parser::Node;
use crate::parser::SpanToken;
use util::parser::Parser;
use util::tree_vec::Tree;

pub(super) fn operator<'a>(
    left_node: Tree<Node<'a>>,
    infix: SpanToken<'a>,
    tokens: &mut Parser<impl Iterator<Item = SpanToken<'a>>>,
) -> Option<Tree<Node<'a>>> {
    if let Token::Operator(operator) = infix.token {
        let (operator, precedence) = match operator {
            Operator::Star => (BinaryOp::Mult, 80),
            Operator::Slash => (BinaryOp::Div, 80),
            Operator::DoubleSlash => (BinaryOp::FlrDiv, 80),
            Operator::Percent => (BinaryOp::Mod, 80),
            Operator::Plus => (BinaryOp::Add, 70),
            Operator::Minus => (BinaryOp::Sub, 70),
            Operator::PlusPlus => (BinaryOp::Concat, 70),
            Operator::DoubleEqual => (BinaryOp::Eq, 60),
            Operator::NotEqual => (BinaryOp::NotEq, 60),
            Operator::Less => (BinaryOp::Lt, 60),
            Operator::LessEqual => (BinaryOp::Lte, 60),
            Operator::Greater => (BinaryOp::Gt, 60),
            Operator::GreaterEqual => (BinaryOp::Gte, 60),
            Operator::Amp => (BinaryOp::And, 50),
            Operator::DoubleAmp => (BinaryOp::LazyAnd, 50),
            Operator::Pipe => (BinaryOp::Or, 40),
            Operator::DoublePipe => (BinaryOp::LazyOr, 40),
            Operator::DoubleQuestion => (BinaryOp::NullOr, 30),
            _ => return None,
        };
        let left_span = left_node.content.span;
        let mut children = left_node.into_tree_vec();
        let right = tokens.partial_parse(precedence);
        let right_span = right.content.span;
        children.push(tokens.partial_parse(precedence));
        Some(Tree {
            content: Node {
                span: left_span.up_to(right_span),
                node: NodeType::Binary(operator),
                unpack: false,
            },
            children,
        })
    } else {
        None
    }
}