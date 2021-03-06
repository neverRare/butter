use crate::lexer::Bracket;
use crate::lexer::Opening;
use crate::lexer::Operator;
use crate::lexer::Token;
use crate::parser::ast::AstType;
use crate::parser::bracket::BracketFragment;
use crate::parser::bracket::BracketSyntax;
use crate::parser::error::ErrorType;
use crate::parser::error::ExpectedToken;
use crate::parser::error_start;
use crate::parser::node_type::Binary;
use crate::parser::node_type::NodeType;
use crate::parser::parenthesis::field_shortcut;
use crate::parser::parenthesis::ParenthesisFragment;
use crate::parser::parenthesis::ParenthesisSyntax;
use crate::parser::AstResult;
use crate::parser::Node;
use crate::parser::Parser;
use std::iter::once;
use util::aggregate_error;
use util::join_trees;
use util::span::span_from_spans;
use util::tree_vec::Tree;
use util::tree_vec::TreeVec;

pub(super) fn operator<'a>(
    parser: &mut Parser<'a>,
    left: AstResult<'a>,
    span: &'a str,
    operator: Operator,
) -> AstResult<'a> {
    match operator {
        Operator::Dot => property_access(parser, left, span, NodeType::Property),
        Operator::LeftArrow => assign(parser, left),
        Operator::Question => question(parser, left, span),
        operator => expr_operator(parser, left, operator),
    }
}
fn expr_operator<'a>(
    parser: &mut Parser<'a>,
    left: AstResult<'a>,
    operator: Operator,
) -> AstResult<'a> {
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
    let (left, right) = aggregate_error(left, parser.parse_expr(precedence))?;
    Ok(Tree {
        content: Node {
            span: span_from_spans(parser.src, left.content.span, right.content.span),
            node: NodeType::Binary(operator),
        },
        children: join_trees![left, right],
    })
}
fn assign<'a>(parser: &mut Parser<'a>, left: AstResult<'a>) -> AstResult<'a> {
    let left = left.and_then(|node| {
        if node.content.node.place() {
            Ok(node)
        } else {
            Err(error_start(node.content.span, ErrorType::NonPlace))
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
    left: AstResult<'a>,
    span: &'a str,
    node: NodeType,
) -> AstResult<'a> {
    let right = match parser.peek_token() {
        Some(Token::Ident) => {
            let span = parser.next().unwrap().span;
            Ok(Tree::new(Node {
                span,
                node: NodeType::Ident,
            }))
        }
        Some(_) | None => Err(error_start(
            &span[span.len()..],
            ErrorType::NoExpectation(&[ExpectedToken::Ident]),
        )),
    };
    let (left, right) = aggregate_error(left, right)?;
    Ok(Tree {
        content: Node {
            span: span_from_spans(parser.src, left.content.span, right.content.span),
            node,
        },
        children: join_trees![left, right],
    })
}
fn question<'a>(parser: &mut Parser<'a>, left: AstResult<'a>, span: &'a str) -> AstResult<'a> {
    match parser.peek_token() {
        Some(Token::Operator(Operator::Dot)) => {
            property_access(parser, left, span, NodeType::OptionalProperty)
        }
        Some(Token::Bracket(Opening::Open, Bracket::Bracket)) => index_or_slice(parser, left, true),
        Some(_) | None => Err(error_start(
            &span[span.len()..],
            ErrorType::NoExpectation(&[
                ExpectedToken::Operator(Operator::Dot),
                ExpectedToken::Bracket(Opening::Open, Bracket::Bracket),
            ]),
        )),
    }
}
pub(super) fn index_or_slice<'a>(
    parser: &mut Parser<'a>,
    left: AstResult<'a>,
    optional: bool,
) -> AstResult<'a> {
    let right = BracketFragment::parse_rest(parser, AstType::Expr, true).map(|bracket_fragment| {
        let (node, right_first, right_second) = match (bracket_fragment.syntax, optional) {
            (BracketSyntax::Single(expr), false) => (NodeType::Index, Some(expr), None),
            (BracketSyntax::Single(expr), true) => (NodeType::OptionalIndex, Some(expr), None),
            (BracketSyntax::Range(first, range_type, second), false) => {
                (NodeType::Slice(range_type), first, second)
            }
            (BracketSyntax::Range(first, range_type, second), true) => {
                (NodeType::OptionalSlice(range_type), first, second)
            }
            _ => unreachable!(),
        };
        (
            bracket_fragment.right_bracket_span,
            node,
            right_first,
            right_second,
        )
    });
    let (left, (right_bracket_span, node, right_first, right_second)) =
        aggregate_error(left, right)?;
    let span = span_from_spans(parser.src, left.content.span, right_bracket_span);
    let children = once(left)
        .chain(right_first.into_iter())
        .chain(right_second.into_iter())
        .collect();
    Ok(Tree {
        content: Node { node, span },
        children,
    })
}
pub(super) fn call<'a>(
    parser: &mut Parser<'a>,
    left: AstResult<'a>,
    left_parenthesis_span: &'a str,
) -> AstResult<'a> {
    let right =
        ParenthesisFragment::parse_rest(parser, AstType::Expr, true).map(|parenthesis_fragment| {
            let (arg_node, args) = match parenthesis_fragment.syntax {
                ParenthesisSyntax::Empty => (NodeType::Struct, TreeVec::new()),
                ParenthesisSyntax::SingleIdent(ident) => {
                    (NodeType::Struct, join_trees![field_shortcut(ident)])
                }
                ParenthesisSyntax::Single(expr) => (NodeType::UnnamedArgs, join_trees![expr]),
                ParenthesisSyntax::UnnamedFields(args) => (NodeType::UnnamedArgs, args),
                ParenthesisSyntax::NamedFields(args) => (NodeType::Struct, args),
            };
            (parenthesis_fragment.right_parenthesis_span, arg_node, args)
        });
    let (left, (right_parenthesis_span, arg_node, args)) = aggregate_error(left, right)?;
    let arg_span = span_from_spans(parser.src, left_parenthesis_span, right_parenthesis_span);
    let arg = Tree {
        content: Node {
            span: arg_span,
            node: arg_node,
        },
        children: args,
    };
    let span = span_from_spans(parser.src, left.content.span, right_parenthesis_span);
    Ok(Tree {
        content: Node {
            span,
            node: NodeType::Call,
        },
        children: join_trees![left, arg],
    })
}
