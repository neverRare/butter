use crate::lexer::Keyword;
use crate::lexer::Operator;
use crate::parser::node_type::NodeType;
use crate::parser::node_type::UnaryOp;
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
    Ok(Tree {
        content: Node {
            span: span.up_to(operand.content.span),
            node: NodeType::Unary(UnaryOp::Clone),
        },
        children: operand.into_tree_vec(),
    })
}
fn unary_operator<'a>(
    span: Span<'a>,
    operator: Operator,
    tokens: &mut Parser<impl Iterator<Item = SpanToken<'a>>>,
) -> ParseResult<'a> {
    let operator = match operator {
        Operator::Plus => UnaryOp::Plus,
        Operator::Minus => UnaryOp::Minus,
        Operator::Bang => UnaryOp::Not,
        Operator::Amp => UnaryOp::Ref,
        operator => unreachable!("expected expression operator, found {:?}", operator),
    };
    let operand = tokens.partial_parse(90)?;
    Ok(Tree {
        content: Node {
            span: span.up_to(operand.content.span),
            node: NodeType::Unary(operator),
        },
        children: operand.into_tree_vec(),
    })
}
fn double_ref<'a>(
    span: Span<'a>,
    tokens: &mut Parser<impl Iterator<Item = SpanToken<'a>>>,
) -> ParseResult<'a> {
    let operand = tokens.partial_parse(90)?;
    let src = span.src();
    let span = span.span();
    debug_assert!(span.len() == 2);
    Ok(Tree {
        content: Node {
            span: Span::from_str(src, &span[..1]),
            node: NodeType::Unary(UnaryOp::Ref),
        },
        children: Tree {
            content: Node {
                span: Span::from_str(src, &span[1..]),
                node: NodeType::Unary(UnaryOp::Ref),
            },
            children: operand.into_tree_vec(),
        }
        .into_tree_vec(),
    })
}
