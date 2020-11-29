use crate::lexer::Keyword;
use crate::lexer::Operator;
use crate::parser::node_type::NodeType;
use crate::parser::node_type::Unary;
use crate::parser::Node;
use crate::parser::ParseResult;
use crate::parser::Parser;
use util::join_trees;
use util::span::span_from_spans;
use util::tree_vec::Tree;

pub(super) fn operator<'a>(
    parser: &mut Parser<'a>,
    span: &'a str,
    operator: Operator,
) -> ParseResult<'a> {
    match operator {
        Operator::Plus | Operator::Minus | Operator::Bang | Operator::Amp => {
            unary_operator(parser, span, operator)
        }
        Operator::DoubleAmp => double_ref(parser, span),
        Operator::RightThickArrow => todo!(),
        operator => panic!("expected prefix operator, found: {:?}", operator),
    }
}
pub(super) fn keyword<'a>(
    parser: &mut Parser<'a>,
    span: &'a str,
    keyword: Keyword,
) -> ParseResult<'a> {
    match keyword {
        Keyword::True | Keyword::False | Keyword::Null => Ok(Tree::new(Node {
            span,
            node: keyword_literal(keyword),
        })),
        Keyword::Clone => clone(parser, span),
        Keyword::Else | Keyword::In => panic!("expected prefix keyword, found {:?}", keyword),
        _ => todo!(),
    }
}
fn keyword_literal(keyword: Keyword) -> NodeType {
    match keyword {
        Keyword::True => NodeType::True,
        Keyword::False => NodeType::False,
        Keyword::Null => NodeType::Null,
        keyword => panic!("expected keyword literal, found {:?}", keyword),
    }
}
fn clone<'a>(parser: &mut Parser<'a>, span: &'a str) -> ParseResult<'a> {
    let operand = parser.parse_expr(90)?;
    Ok(Tree {
        content: Node {
            span: span_from_spans(parser.src, span, operand.content.span),
            node: NodeType::Unary(Unary::Clone),
        },
        children: join_trees![operand],
    })
}
fn unary_operator<'a>(
    parser: &mut Parser<'a>,
    span: &'a str,
    operator: Operator,
) -> ParseResult<'a> {
    let operator = match operator {
        Operator::Plus => Unary::Plus,
        Operator::Minus => Unary::Minus,
        Operator::Bang => Unary::Not,
        Operator::Amp => Unary::Ref,
        operator => panic!("expected expression operator, found {:?}", operator),
    };
    let operand = parser.parse_expr(90)?;
    Ok(Tree {
        content: Node {
            span: span_from_spans(parser.src, span, operand.content.span),
            node: NodeType::Unary(operator),
        },
        children: join_trees![operand],
    })
}
fn double_ref<'a>(parser: &mut Parser<'a>, span: &'a str) -> ParseResult<'a> {
    let operand = parser.parse_expr(90)?;
    assert!(span == "&&");
    Ok(Tree {
        content: Node {
            span: &span[..1],
            node: NodeType::Unary(Unary::Ref),
        },
        children: join_trees![Tree {
            content: Node {
                span: &span[1..],
                node: NodeType::Unary(Unary::Ref),
            },
            children: join_trees![operand],
        }],
    })
}
