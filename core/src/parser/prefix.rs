use crate::lexer::Keyword;
use crate::lexer::Operator;
use crate::parser::error::ErrorType;
use crate::parser::node_type::NodeType;
use crate::parser::node_type::Unary;
use crate::parser::Error;
use crate::parser::Node;
use crate::parser::ParseResult;
use crate::parser::SpanToken;
use util::parser::Parser;
use util::span::Span;
use util::tree_vec::Tree;

pub(super) fn operator<'a>(
    span: Span<'a>,
    operator: Operator,
    tokens: &mut Parser<impl Iterator<Item = SpanToken<'a>>>,
) -> ParseResult<'a> {
    match operator {
        Operator::Plus | Operator::Minus | Operator::Bang | Operator::Amp => {
            unary_operator(span, operator, tokens)
        }
        Operator::DoubleAmp => double_ref(span, tokens),
        _ => todo!(),
    }
}
pub(super) fn keyword<'a>(
    span: Span<'a>,
    keyword: Keyword,
    tokens: &mut Parser<impl Iterator<Item = SpanToken<'a>>>,
) -> ParseResult<'a> {
    match keyword {
        Keyword::True | Keyword::False | Keyword::Null => Ok(Tree::new(Node {
            span,
            node: keyword_literal(keyword),
        })),
        Keyword::Clone => clone(span, tokens),
        _ => todo!(),
    }
}
fn keyword_literal(keyword: Keyword) -> NodeType {
    match keyword {
        Keyword::True => NodeType::True,
        Keyword::False => NodeType::False,
        Keyword::Null => NodeType::Null,
        keyword => unreachable!("expected keyword literal, found {:?}", keyword),
    }
}
fn clone<'a>(
    span: Span<'a>,
    tokens: &mut Parser<impl Iterator<Item = SpanToken<'a>>>,
) -> ParseResult<'a> {
    let operand = tokens.partial_parse(90)?;
    if operand.content.node.expr() {
        Ok(Tree {
            content: Node {
                span: span.up_to(operand.content.span),
                node: NodeType::Unary(Unary::Clone),
            },
            children: operand.into_tree_vec(),
        })
    } else {
        Err(vec![Error {
            span: operand.content.span,
            error: ErrorType::NonExprOperand,
        }])
    }
}
fn unary_operator<'a>(
    span: Span<'a>,
    operator: Operator,
    tokens: &mut Parser<impl Iterator<Item = SpanToken<'a>>>,
) -> ParseResult<'a> {
    let operator = match operator {
        Operator::Plus => Unary::Plus,
        Operator::Minus => Unary::Minus,
        Operator::Bang => Unary::Not,
        Operator::Amp => Unary::Ref,
        operator => unreachable!("expected expression operator, found {:?}", operator),
    };
    let operand = tokens.partial_parse(90)?;
    if operand.content.node.expr() {
        Ok(Tree {
            content: Node {
                span: span.up_to(operand.content.span),
                node: NodeType::Unary(operator),
            },
            children: operand.into_tree_vec(),
        })
    } else {
        Err(vec![Error {
            span: operand.content.span,
            error: ErrorType::NonExprOperand,
        }])
    }
}
fn double_ref<'a>(
    span: Span<'a>,
    tokens: &mut Parser<impl Iterator<Item = SpanToken<'a>>>,
) -> ParseResult<'a> {
    let operand = tokens.partial_parse(90)?;
    if operand.content.node.expr() {
        let src = span.src();
        let span = span.span();
        debug_assert!(span.len() == 2);
        Ok(Tree {
            content: Node {
                span: Span::from_str(src, &span[..1]),
                node: NodeType::Unary(Unary::Ref),
            },
            children: Tree {
                content: Node {
                    span: Span::from_str(src, &span[1..]),
                    node: NodeType::Unary(Unary::Ref),
                },
                children: operand.into_tree_vec(),
            }
            .into_tree_vec(),
        })
    } else {
        Err(vec![Error {
            span: operand.content.span,
            error: ErrorType::NonExprOperand,
        }])
    }
}
