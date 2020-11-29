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
use util::join_trees;
use util::span::span_from_spans;
use util::tree_vec::Tree;

pub(super) fn operator<'a>(
    parser: &mut Parser<'a>,
    left: ParseResult<'a>,
    span: &'a str,
    operator: Operator,
) -> ParseResult<'a> {
    match operator {
        Operator::Dot => property_access(parser, left, span),
        Operator::LeftArrow => assign(parser, left),
        Operator::Question => todo!(),
        operator => expr_operator(parser, left, operator),
    }
}
fn expr_operator<'a>(
    parser: &mut Parser<'a>,
    left: ParseResult<'a>,
    operator: Operator,
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
        operator => panic!("expected expression operator, found {:?}", operator),
    };
    let (left, right) = aggregate_error(left.and_then(assert_expr), parser.parse_expr(precedence))?;
    Ok(Tree {
        content: Node {
            span: span_from_spans(parser.src, left.content.span, right.content.span),
            node: NodeType::Binary(operator),
        },
        children: join_trees![left, right],
    })
}
fn assign<'a>(parser: &mut Parser<'a>, left: ParseResult<'a>) -> ParseResult<'a> {
    let left = left.and_then(|node| {
        if node.content.node.place() {
            Ok(node)
        } else {
            Err(vec![Error {
                span: node.content.span,
                error: ErrorType::NonPlace,
            }])
        }
    });
    let (left, right) = aggregate_error(left, parser.parse_expr(19))?;
    Ok(Tree {
        content: Node {
            span: span_from_spans(parser.src, left.content.span, right.content.span),
            node: NodeType::Assign,
        },
        children: join_trees![left, right],
    })
}
fn property_access<'a>(
    parser: &mut Parser<'a>,
    left: ParseResult<'a>,
    span: &'a str,
) -> ParseResult<'a> {
    let right = if let Some(SpanToken {
        span: _,
        token: Token::Ident,
    }) = parser.peek()
    {
        let span = parser.next().unwrap().span;
        Ok(Tree::new(Node {
            span,
            node: NodeType::Ident,
        }))
    } else {
        Err(vec![Error {
            span: &span[span.len()..],
            error: ErrorType::NoIdent,
        }])
    };
    let (left, right) = aggregate_error(left.and_then(assert_expr), right)?;
    Ok(Tree {
        content: Node {
            span: span_from_spans(parser.src, left.content.span, right.content.span),
            node: NodeType::Property,
        },
        children: join_trees![left, right],
    })
}
