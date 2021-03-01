use crate::lexer::Bracket;
use crate::lexer::Opening;
use crate::lexer::Operator;
use crate::lexer::Separator;
use crate::lexer::Token;
use crate::parser::error_start;
use crate::parser::node_type::RangeType;
use crate::parser::ErrorType;
use crate::parser::Node;
use crate::parser::NodeType;
use crate::parser::Parser;
use crate::parser::ParserResult;
use crate::parser::TokenKind;
use util::iter::PeekableIterator;
use util::join_trees;
use util::parser::ParserIter;
use util::span::span_from_spans;
use util::tree_vec::Tree;
use util::tree_vec::TreeVec;

static EXPECTED_TOKEN: &[TokenKind] = &[
    TokenKind::Operator(Operator::Star),
    TokenKind::Bracket(Opening::Close, Bracket::Bracket),
    TokenKind::Separator(Separator::Comma),
    TokenKind::Operator(Operator::DoubleDot),
    TokenKind::Operator(Operator::DotLess),
    TokenKind::Operator(Operator::GreaterDot),
    TokenKind::Operator(Operator::GreaterLess),
];
enum BracketSyntax<'a> {
    Empty,
    Single(Tree<Node<'a>>),
    Multiple(TreeVec<Node<'a>>),
    Range(RangeType, TreeVec<Node<'a>>),
}
struct BracketFragment<'a> {
    syntax: BracketSyntax<'a>,
    right_bracket_span: &'a str,
}
impl<'a> BracketFragment<'a> {
    fn parse_rest(parser: &mut Parser<'a>) -> ParserResult<'a, Self> {
        let first = parser.parse_optional_expr(0)?;
        let token = parser.peek();
        match token.map(|token| token.token) {
            Some(Token::Bracket(Opening::Close, Bracket::Bracket)) => {
                let right_bracket_span = parser.next().unwrap().span;
                let syntax = match first {
                    Some(expr) => BracketSyntax::Single(expr),
                    None => BracketSyntax::Empty,
                };
                Ok(Self {
                    syntax,
                    right_bracket_span,
                })
            }
            Some(Token::Separator(Separator::Comma)) | Some(Token::Operator(Operator::Star)) => {
                let token = token.unwrap();
                let mut elements = TreeVec::new();
                if let Token::Separator(Separator::Comma) = token.token {
                    match first {
                        Some(expr) => elements.push(expr),
                        None => {
                            let comma_span = parser.next().unwrap().span;
                            return Err(error_start(&comma_span[..0], ErrorType::NoExpr));
                        }
                    }
                } else {
                    debug_assert!(first.is_none());
                }
                while let Some(Token::Separator(Separator::Comma)) = parser.peek_token() {
                    parser.next();
                    if let Some(Token::Operator(Operator::Star)) = parser.peek_token() {
                        let token_span = parser.next().unwrap().span;
                        let expr = parser.partial_parse(0)?;
                        elements.push(Tree {
                            content: Node {
                                span: span_from_spans(parser.src, token_span, expr.content.span),
                                node: NodeType::Splat,
                            },
                            children: join_trees![expr],
                        });
                    } else {
                        match parser.parse_optional_expr(0)? {
                            Some(expr) => elements.push(expr),
                            None => break,
                        }
                    }
                }
                let right_bracket_span = get_right_bracket_span(parser, &EXPECTED_TOKEN[1..2])?;
                Ok(Self {
                    syntax: BracketSyntax::Multiple(elements),
                    right_bracket_span,
                })
            }
            Some(Token::Operator(Operator::DoubleDot))
            | Some(Token::Operator(Operator::DotLess))
            | Some(Token::Operator(Operator::GreaterDot))
            | Some(Token::Operator(Operator::GreaterLess)) => {
                let operator = {
                    if let Token::Operator(operator) = parser.next().unwrap().token {
                        operator
                    } else {
                        unreachable!()
                    }
                };
                let second = parser.parse_optional_expr(0)?;
                let right_bracket_span = get_right_bracket_span(parser, &EXPECTED_TOKEN[1..1])?;
                let range_type = match (&first, operator, &second) {
                    (Some(_), operator, Some(_)) => match operator {
                        Operator::DoubleDot => RangeType::Inclusive,
                        Operator::DotLess => RangeType::InclusiveExclusive,
                        Operator::GreaterDot => RangeType::ExclusiveInclusive,
                        Operator::GreaterLess => RangeType::Exclusive,
                        _ => unreachable!(),
                    },
                    (Some(_), operator, None) => match operator {
                        Operator::DoubleDot | Operator::DotLess => RangeType::FromInclusive,
                        Operator::GreaterDot | Operator::GreaterLess => RangeType::FromExclusive,
                        _ => unreachable!(),
                    },
                    (None, operator, Some(_)) => match operator {
                        Operator::DoubleDot | Operator::GreaterDot => RangeType::ToInclusive,
                        Operator::DotLess | Operator::GreaterLess => RangeType::ToExclusive,
                        _ => unreachable!(),
                    },
                    (None, _, None) => RangeType::Full,
                };
                let tree_vec = first.into_iter().chain(second.into_iter()).collect();
                Ok(Self {
                    syntax: BracketSyntax::Range(range_type, tree_vec),
                    right_bracket_span,
                })
            }
            Some(_) | None => {
                let expected = match first {
                    Some(_) => &EXPECTED_TOKEN[1..],
                    None => EXPECTED_TOKEN,
                };
                let span = match token {
                    Some(token) => &token.span[..0],
                    None => &parser.src[parser.src.len()..],
                };
                Err(error_start(span, ErrorType::NoExpectation(expected)))
            }
        }
    }
}
fn get_right_bracket_span<'a>(
    parser: &mut Parser<'a>,
    expectation: &'static [TokenKind],
) -> ParserResult<'a, &'a str> {
    let token = parser.peek();
    if let Some(Token::Bracket(Opening::Close, Bracket::Bracket)) = token.map(|token| token.token) {
        Ok(parser.next().unwrap().span)
    } else {
        let span = match token {
            Some(token) => &token.span[..0],
            None => &parser.src[parser.src.len()..],
        };
        Err(error_start(span, ErrorType::NoExpectation(expectation)))
    }
}
