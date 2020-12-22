use crate::lexer::Keyword;
use crate::lexer::Operator;
use crate::lexer::Token;
use crate::parser::node_type::NodeType;
use crate::parser::node_type::Unary;
use crate::parser::Node;
use crate::parser::ParseResult;
use crate::parser::Parser;
use util::iter::PeekableIterator;
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
        Keyword::If => todo!(),
        Keyword::For => todo!(),
        Keyword::Loop => todo!(),
        Keyword::While => todo!(),
        Keyword::Break => parse_break(parser, span),
        Keyword::Continue => parse_continue(parser, span),
        Keyword::Return => parse_return(parser, span),
        _ => panic!("expected prefix keyword, found {:?}", keyword),
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
fn parse_continue<'a>(parser: &mut Parser<'a>, span: &'a str) -> ParseResult<'a> {
    let label = optional_label(parser);
    Ok(Tree {
        content: Node {
            span: match &label {
                Some(label) => span_from_spans(parser.src, span, label.content.span),
                None => span,
            },
            node: NodeType::Continue,
        },
        children: label.into_iter().collect(),
    })
}
fn parse_break<'a>(parser: &mut Parser<'a>, span: &'a str) -> ParseResult<'a> {
    let label = optional_label(parser);
    let expr =
        if let Some(Token::Operator(Operator::Equal)) = parser.peek().map(|token| token.token) {
            parser.next();
            Some(parser.parse_expr(10)?)
        } else {
            None
        };
    let node = match &expr {
        Some(_) => NodeType::BreakWithExpr,
        None => NodeType::Break,
    };
    let span = if let Some(expr) = &expr {
        span_from_spans(parser.src, span, expr.content.span)
    } else if let Some(label) = &label {
        span_from_spans(parser.src, span, label.content.span)
    } else {
        span
    };
    Ok(Tree {
        content: Node { span, node },
        children: label.into_iter().chain(expr.into_iter()).collect(),
    })
}
fn parse_return<'a>(parser: &mut Parser<'a>, span: &'a str) -> ParseResult<'a> {
    let expr = parser.parse_optional_expr(10)?;
    Ok(Tree {
        content: Node {
            span: match &expr {
                Some(label) => span_from_spans(parser.src, span, label.content.span),
                None => span,
            },
            node: NodeType::Return,
        },
        children: expr.into_iter().collect(),
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
fn optional_label<'a>(parser: &mut Parser<'a>) -> Option<Tree<Node<'a>>> {
    match parser.peek()?.token {
        Token::Keyword(_) | Token::Ident => {
            let token = parser.next().unwrap();
            Some(Tree::new(Node {
                span: token.span,
                node: NodeType::Label,
            }))
        }
        _ => None,
    }
}
