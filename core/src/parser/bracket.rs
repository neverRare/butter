use crate::lexer::Bracket;
use crate::lexer::Opening;
use crate::lexer::Operator;
use crate::lexer::Separator;
use crate::lexer::Token;
use crate::parser::ast::Ast;
use crate::parser::ast::AstType;
use crate::parser::ast::AstVec;
use crate::parser::error_start;
use crate::parser::node_type::RangeType;
use crate::parser::ErrorType;
use crate::parser::ExpectedToken;
use crate::parser::Node;
use crate::parser::NodeType;
use crate::parser::Parser;
use crate::parser::ParserResult;
use util::iter::PeekableIterator;
use util::join_trees;
use util::parser::ParserIter;
use util::span::span_from_spans;
use util::tree_vec::Tree;
use util::tree_vec::TreeVec;

static EXPECTED_TOKEN: &[ExpectedToken] = &[
    ExpectedToken::Operator(Operator::Star),
    ExpectedToken::Bracket(Opening::Close, Bracket::Bracket),
    ExpectedToken::Separator(Separator::Comma),
    ExpectedToken::Operator(Operator::DoubleDot),
    ExpectedToken::Operator(Operator::DotLess),
    ExpectedToken::Operator(Operator::GreaterDot),
    ExpectedToken::Operator(Operator::GreaterLess),
];
pub(super) enum BracketSyntax<'a> {
    Empty,
    Single(Ast<'a>),
    Multiple(AstVec<'a>),
    Range(Option<Ast<'a>>, RangeType, Option<Ast<'a>>),
}
pub(super) struct BracketFragment<'a> {
    pub(super) syntax: BracketSyntax<'a>,
    pub kind: AstType,
    pub right_bracket_span: &'a str,
}
impl<'a> BracketFragment<'a> {
    pub(super) fn parse_rest(
        parser: &mut Parser<'a>,
        kind: AstType,
        index_or_slice: bool,
    ) -> ParserResult<'a, Self> {
        if index_or_slice {
            assert!(kind.is_expr());
        }
        // TODO: aggregate error as possible
        let first = parser.parse_optional(0, kind)?;
        let token = parser.peek();
        match token.map(|token| token.token) {
            Some(Token::Bracket(Opening::Close, Bracket::Bracket)) => {
                let right_bracket_span = parser.next().unwrap().span;
                let (kind, syntax) = match first {
                    Some(ast) => (ast.kind, BracketSyntax::Single(ast.ast)),
                    None if index_or_slice => {
                        return Err(error_start(&right_bracket_span[..0], ErrorType::NoExpr));
                    }
                    None => (kind, BracketSyntax::Empty),
                };
                Ok(Self {
                    syntax,
                    kind,
                    right_bracket_span,
                })
            }
            Some(Token::Separator(Separator::Comma)) | Some(Token::Operator(Operator::Star))
                if !index_or_slice =>
            {
                let token = token.unwrap();
                let mut elements = TreeVec::new();
                let mut kind = first.as_ref().map(|ast| ast.kind).unwrap_or(kind);
                if let Token::Separator(Separator::Comma) = token.token {
                    match first {
                        Some(ast) => elements.push(ast.ast),
                        None => {
                            let comma_span = parser.next().unwrap().span;
                            return Err(error_start(&comma_span[..0], ErrorType::NoExpr));
                        }
                    }
                } else {
                    debug_assert!(first.is_none());
                }
                let mut star_before = false;
                while let Some(Token::Separator(Separator::Comma)) = parser.peek_token() {
                    parser.next();
                    if let Some(Token::Bracket(Opening::Close, Bracket::Bracket)) =
                        parser.peek_token()
                    {
                        break;
                    }
                    if let Some(Token::Operator(Operator::Star)) = parser.peek_token() {
                        let star_span = parser.next().unwrap().span;
                        if star_before {
                            if !kind.is_expr() {
                                return Err(error_start(star_span, ErrorType::RestAfterRest));
                            } else {
                                kind = AstType::Expr;
                            }
                        }
                        star_before = true;
                        let ast = parser.parse(0, &kind)?;
                        kind = ast.kind;
                        elements.push(Tree {
                            content: Node {
                                span: span_from_spans(parser.src, star_span, ast.ast.content.span),
                                node: NodeType::SplatOrRest,
                            },
                            children: join_trees![ast.ast],
                        });
                    } else {
                        let ast = parser.parse(0, &kind)?;
                        kind = ast.kind;
                        elements.push(ast.ast);
                    }
                }
                let right_bracket_span = parser.get_span(
                    Token::Bracket(Opening::Close, Bracket::Bracket),
                    &EXPECTED_TOKEN[1..2],
                )?;
                Ok(Self {
                    syntax: BracketSyntax::Multiple(elements),
                    kind,
                    right_bracket_span,
                })
            }
            Some(Token::Operator(Operator::DoubleDot))
            | Some(Token::Operator(Operator::DotLess))
            | Some(Token::Operator(Operator::GreaterDot))
            | Some(Token::Operator(Operator::GreaterLess))
                if kind.is_expr() =>
            {
                let first = first.map(|ast| ast.ast);
                let operator = {
                    if let Token::Operator(operator) = parser.next().unwrap().token {
                        operator
                    } else {
                        unreachable!()
                    }
                };
                let second = parser.parse_optional_expr(0)?;
                let right_bracket_span = parser.get_span(
                    Token::Bracket(Opening::Close, Bracket::Bracket),
                    &EXPECTED_TOKEN[1..1],
                )?;
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
                Ok(Self {
                    kind: AstType::Expr,
                    syntax: BracketSyntax::Range(first, range_type, second),
                    right_bracket_span,
                })
            }
            Some(_) | None => {
                let expected = match (first, kind.is_expr()) {
                    (Some(_), true) => &EXPECTED_TOKEN[1..],
                    (Some(_), false) => &EXPECTED_TOKEN[1..3],
                    (None, true) => EXPECTED_TOKEN,
                    (None, false) => &EXPECTED_TOKEN[..3],
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
