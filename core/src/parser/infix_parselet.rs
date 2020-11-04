use crate::lexer::Operator;
use crate::parser::error::ErrorType;
use crate::parser::node_type::BinaryOp;
use crate::parser::node_type::NodeType;
use crate::parser::Error;
use crate::parser::Node;
use crate::parser::ParseResult;
use crate::parser::SpanToken;
use util::aggregate_error;
use util::parser::Parser;
use util::tree_vec::Tree;

pub(super) fn operator<'a>(
    left_node: ParseResult<'a>,
    operator: Operator,
    tokens: &mut Parser<impl Iterator<Item = SpanToken<'a>>>,
) -> ParseResult<'a> {
    match operator {
        Operator::Dot => todo!(),
        Operator::LeftArrow => assign(left_node, tokens),
        operator => expr_operator(left_node, operator, tokens),
    }
}
// TODO: handle operands not being valid expression
fn expr_operator<'a>(
    left_node: ParseResult<'a>,
    operator: Operator,
    tokens: &mut Parser<impl Iterator<Item = SpanToken<'a>>>,
) -> ParseResult<'a> {
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
        operator => unreachable!("expected expression operator, found {:?}", operator),
    };
    let (left_node, right) = aggregate_error(left_node, tokens.partial_parse(precedence))?;
    let left_span = left_node.content.span;
    let mut children = left_node.into_tree_vec();
    let right_span = right.content.span;
    children.push(right);
    Ok(Tree {
        content: Node {
            span: left_span.up_to(right_span),
            node: NodeType::Binary(operator),
        },
        children,
    })
}
fn assign<'a>(
    left_node: ParseResult<'a>,
    tokens: &mut Parser<impl Iterator<Item = SpanToken<'a>>>,
) -> ParseResult<'a> {
    let left_node = left_node.and_then(|node| {
        if node.content.node.place() {
            Err(vec![Error {
                span: node.content.span,
                error: ErrorType::NonPlaceAssign,
            }])
        } else {
            Ok(node)
        }
    });
    let (left_node, right) = aggregate_error(left_node, tokens.partial_parse(19))?;
    let left_span = left_node.content.span;
    let mut children = left_node.into_tree_vec();
    let right_span = right.content.span;
    children.push(right);
    Ok(Tree {
        content: Node {
            span: left_span.up_to(right_span),
            node: NodeType::Assign,
        },
        children,
    })
}
