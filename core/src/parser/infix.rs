use crate::lexer::Operator;
use crate::parser::error::ErrorType;
use crate::parser::node_type::Binary;
use crate::parser::node_type::NodeType;
use crate::parser::Error;
use crate::parser::Node;
use crate::parser::ParseResult;
use crate::parser::Parser;
use crate::parser::SpanToken;
use util::aggregate_error;
use util::iter::PeekableIter;
use util::parser::ParserIter;
use util::tree_vec::Tree;

pub(super) fn operator<'a>(
    left_node: ParseResult<'a>,
    operator: Operator,
    tokens: &mut Parser<impl PeekableIter<Item = SpanToken<'a>>>,
) -> ParseResult<'a> {
    match operator {
        Operator::Dot => todo!(),
        Operator::LeftArrow => assign(left_node, tokens),
        operator => expr_operator(left_node, operator, tokens),
    }
}
fn expr_operator<'a>(
    left_node: ParseResult<'a>,
    operator: Operator,
    tokens: &mut Parser<impl PeekableIter<Item = SpanToken<'a>>>,
) -> ParseResult<'a> {
    let (operator, precedence) = match operator {
        Operator::Star => (Binary::Multiply, 80),
        Operator::Slash => (Binary::Div, 80),
        Operator::DoubleSlash => (Binary::FloorDiv, 80),
        Operator::Percent => (Binary::Mod, 80),
        Operator::Plus => (Binary::Add, 70),
        Operator::Minus => (Binary::Sub, 70),
        Operator::DoublePlus => (Binary::Concatenate, 70),
        Operator::DoubleEqual => (Binary::Eq, 60),
        Operator::NotEqual => (Binary::NotEq, 60),
        Operator::Less => (Binary::Less, 60),
        Operator::LessEqual => (Binary::LessEqual, 60),
        Operator::Greater => (Binary::Greater, 60),
        Operator::GreaterEqual => (Binary::GreaterEqual, 60),
        Operator::Amp => (Binary::And, 50),
        Operator::DoubleAmp => (Binary::LazyAnd, 50),
        Operator::Pipe => (Binary::Or, 40),
        Operator::DoublePipe => (Binary::LazyOr, 40),
        Operator::DoubleQuestion => (Binary::NullOr, 30),
        operator => unreachable!("expected expression operator, found {:?}", operator),
    };
    let right = tokens.partial_parse(precedence).and_then(|node| {
        if node.content.node.expr() {
            Ok(node)
        } else {
            Err(vec![Error {
                span: node.content.span,
                error: ErrorType::NonExprOperand,
            }])
        }
    });
    let (left_node, right) = aggregate_error(left_node, right)?;
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
    tokens: &mut Parser<impl PeekableIter<Item = SpanToken<'a>>>,
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
    let right = tokens.partial_parse(19).and_then(|node| {
        if node.content.node.expr() {
            Ok(node)
        } else {
            Err(vec![Error {
                span: node.content.span,
                error: ErrorType::NonExprOperand,
            }])
        }
    });
    let (left_node, right) = aggregate_error(left_node, right)?;
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
