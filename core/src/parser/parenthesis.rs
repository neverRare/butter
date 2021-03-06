use crate::lexer::Bracket;
use crate::lexer::Opening;
use crate::lexer::Operator;
use crate::lexer::Separator;
use crate::lexer::Token;
use crate::parser::ast::AstVec;
use crate::parser::error_start;
use crate::parser::prefix;
use crate::parser::Ast;
use crate::parser::AstType;
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

pub(super) enum ParenthesisSyntax<'a> {
    Empty,
    Single(Ast<'a>),
    NamedFields(AstVec<'a>),
    UnnamedFields(AstVec<'a>),
}
pub(super) struct ParenthesisFragment<'a> {
    pub(super) syntax: ParenthesisSyntax<'a>,
    pub kind: AstType,
    pub have_splat_or_rest: bool,
    pub right_parenthesis_span: &'a str,
}
impl<'a> ParenthesisFragment<'a> {
    pub fn parse_rest(parser: &mut Parser<'a>, kind: AstType, arg: bool) -> ParserResult<'a, Self> {
        if arg {
            assert!(kind.is_expr());
        }
        todo!()
    }
}
struct FieldFragment<'a> {
    ast: Ast<'a>,
    field_kind: FieldType,
    kind: AstType,
}
impl<'a> FieldFragment<'a> {
    pub fn parse(
        parser: &mut Parser<'a>,
        kind: AstType,
        field_kind: FieldTypeRequest,
        star_before: bool,
    ) -> ParserResult<'a, Self> {
        if !field_kind.is_named() {
            assert!(kind.is_expr());
        }
        let token = parser.peek();
        match token.map(|token| token.token) {
            Some(Token::Ident) if field_kind.is_named() => {
                let ident = parser.next().unwrap();
                match parser.peek_token() {
                    Some(Token::Operator(Operator::Equal)) => {
                        parser.next();
                        let ast = parser.parse(0, &kind)?;
                        let name_ast = Tree::new(Node {
                            span: ident.span,
                            node: NodeType::Name,
                        });
                        Ok(Self {
                            ast: Tree {
                                content: Node {
                                    span: span_from_spans(
                                        parser.src,
                                        ident.span,
                                        ast.ast.content.span,
                                    ),
                                    node: NodeType::Field,
                                },
                                children: join_trees![name_ast, ast.ast],
                            },
                            field_kind: FieldType::Named,
                            kind: ast.kind,
                        })
                    }
                    Some(Token::Separator(Separator::Comma))
                    | Some(Token::Bracket(Opening::Close, Bracket::Parenthesis)) => {
                        let name_ast = Tree::new(Node {
                            span: ident.span,
                            node: NodeType::Name,
                        });
                        let ident_ast = Tree::new(Node {
                            span: ident.span,
                            node: NodeType::Ident,
                        });
                        Ok(Self {
                            ast: Tree {
                                content: Node {
                                    span: ident.span,
                                    node: NodeType::Field,
                                },
                                children: join_trees![name_ast, ident_ast],
                            },
                            field_kind: FieldType::Named,
                            kind,
                        })
                    }
                    Some(_) | None => {
                        let ast = prefix::ident(parser, ident.span, kind)?;
                        Ok(Self {
                            ast: ast.ast,
                            field_kind: FieldType::Unnamed,
                            kind: ast.kind,
                        })
                    }
                }
            }
            Some(Token::Operator(Operator::Star)) => {
                if !field_kind.is_named() {
                    let token_span = token.unwrap().span;
                    return Err(error_start(
                        &token_span[token_span.len()..],
                        ErrorType::NoExpr,
                    ));
                }
                let star_span = parser.next().unwrap().span;
                let kind = if star_before {
                    if !kind.is_expr() {
                        return Err(error_start(star_span, ErrorType::RestAfterRest));
                    } else {
                        AstType::Expr
                    }
                } else {
                    kind
                };
                let ast = parser.parse(0, &kind)?;
                Ok(Self {
                    ast: Tree {
                        content: Node {
                            span: span_from_spans(parser.src, star_span, ast.ast.content.span),
                            node: NodeType::SplatOrRest,
                        },
                        children: join_trees![ast.ast],
                    },
                    field_kind: FieldType::SplatOrRest,
                    kind: ast.kind,
                })
            }
            Some(_) | None if field_kind.is_nameless() => {
                let ast = parser.parse_expr(0)?;
                Ok(Self {
                    ast,
                    field_kind: FieldType::Unnamed,
                    kind: AstType::Expr,
                })
            }
            Some(_) | None => {
                let token_span = token.unwrap().span;
                Err(error_start(
                    &token_span[token_span.len()..],
                    ErrorType::NoExpectation(&[
                        ExpectedToken::Ident,
                        ExpectedToken::Operator(Operator::Star),
                    ]),
                ))
            }
        }
    }
}
enum FieldType {
    Unnamed,
    Named,
    SplatOrRest,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum FieldTypeRequest {
    Named,
    Unnamed,
    Either,
}
impl FieldTypeRequest {
    pub fn is_named(self) -> bool {
        matches!(self, Self::Named | Self::Either)
    }
    pub fn is_nameless(self) -> bool {
        matches!(self, Self::Unnamed | Self::Either)
    }
}
