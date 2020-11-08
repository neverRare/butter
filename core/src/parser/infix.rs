use crate::lexer::Operator;
use crate::lexer::Token;
use crate::parser::assert_expr;
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
use util::span::Span;
use util::tree_vec::Tree;

pub(super) fn operator<'a>(
    left: ParseResult<'a>,
    span: Span<'a>,
    operator: Operator,
    tokens: &mut Parser<impl PeekableIter<Item = SpanToken<'a>>>,
) -> ParseResult<'a> {
    match operator {
        Operator::Dot => property_access(left, span, tokens),
        Operator::LeftArrow => assign(left, tokens),
        operator => expr_operator(left, operator, tokens),
    }
}
fn expr_operator<'a>(
    left: ParseResult<'a>,
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
        Operator::DoubleEqual => (Binary::Equal, 60),
        Operator::NotEqual => (Binary::NotEqual, 60),
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
    let (left, right) = aggregate_error(left.and_then(assert_expr), tokens.parse_expr(precedence))?;
    let left_span = left.content.span;
    let mut children = left.into_tree_vec();
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
    left: ParseResult<'a>,
    tokens: &mut Parser<impl PeekableIter<Item = SpanToken<'a>>>,
) -> ParseResult<'a> {
    let left_node = left.and_then(|node| {
        if node.content.node.place() {
            Ok(node)
        } else {
            Err(vec![Error {
                span: node.content.span,
                error: ErrorType::NonPlaceAssign,
            }])
        }
    });
    let (left, right) = aggregate_error(left_node, tokens.parse_expr(19))?;
    let left_span = left.content.span;
    let mut children = left.into_tree_vec();
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
fn property_access<'a>(
    left: ParseResult<'a>,
    span: Span<'a>,
    tokens: &mut Parser<impl PeekableIter<Item = SpanToken<'a>>>,
) -> ParseResult<'a> {
    let right = if let Some(SpanToken {
        span: _,
        token: Token::Ident,
    }) = tokens.peek()
    {
        let span = tokens.next().unwrap().span;
        Ok(Tree::new(Node {
            span,
            node: NodeType::Ident,
        }))
    } else {
        Err(vec![Error {
            span,
            error: ErrorType::NonIdent,
        }])
    };
    let (left, right) = aggregate_error(left.and_then(assert_expr), right)?;
    let left_span = left.content.span;
    let mut children = left.into_tree_vec();
    let right_span = right.content.span;
    children.push(right);
    Ok(Tree {
        content: Node {
            span: left_span.up_to(right_span),
            node: NodeType::Property,
        },
        children,
    })
}
