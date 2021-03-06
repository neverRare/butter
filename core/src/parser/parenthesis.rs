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
    SingleIdent(&'a str),
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
        if let Some(Token::Bracket(Opening::Close, Bracket::Parenthesis)) = parser.peek_token() {
            let right_parenthesis_span = parser.next().unwrap().span;
            return Ok(Self {
                syntax: ParenthesisSyntax::Empty,
                kind,
                have_splat_or_rest: false,
                right_parenthesis_span,
            });
        }
        let first = FieldFragment::parse(parser, kind, FieldTypeRequest::Either, false)?;
        if let Some(Token::Bracket(Opening::Close, Bracket::Parenthesis)) = parser.peek_token() {
            let right_parenthesis_span = parser.next().unwrap().span;
            let have_splat_or_rest = matches!(first.syntax, FieldSyntax::SplatOrRest(_));
            let kind = first.kind;
            let syntax = match first.syntax {
                FieldSyntax::Ident(ident) => ParenthesisSyntax::SingleIdent(ident),
                FieldSyntax::Unnamed(ast) => ParenthesisSyntax::Single(ast),
                FieldSyntax::Named(ast) => ParenthesisSyntax::NamedFields(join_trees![ast]),
                FieldSyntax::SplatOrRest(ast) => ParenthesisSyntax::NamedFields(join_trees![ast]),
            };
            return Ok(Self {
                syntax,
                kind,
                have_splat_or_rest,
                right_parenthesis_span,
            });
        }
        let mut star_before = matches!(first.syntax, FieldSyntax::SplatOrRest(_));
        let mut fields = match first.syntax {
            FieldSyntax::Ident(ident) => join_trees![field_shortcut(ident)],
            FieldSyntax::Unnamed(ast) if !arg => {
                return Err(error_start(ast.content.span, ErrorType::NotNamed));
            }
            FieldSyntax::Unnamed(ast) | FieldSyntax::Named(ast) | FieldSyntax::SplatOrRest(ast) => {
                join_trees![ast]
            }
        };
        let mut kind = first.kind;
        let mut request = if arg {
            FieldTypeRequest::Either
        } else {
            FieldTypeRequest::Named
        };
        while let Some(Token::Separator(Separator::Comma)) = parser.peek_token() {
            let fragment = FieldFragment::parse(parser, kind, request, star_before)?;
            kind = fragment.kind;
            let field = match fragment.syntax {
                FieldSyntax::Ident(ident) => field_shortcut(ident),
                FieldSyntax::Unnamed(ast) => {
                    debug_assert!(request.is_unnamed());
                    request = FieldTypeRequest::Unnamed;
                    ast
                }
                FieldSyntax::Named(ast) => {
                    debug_assert!(request.is_named());
                    request = FieldTypeRequest::Named;
                    ast
                }
                FieldSyntax::SplatOrRest(ast) => {
                    star_before = true;
                    ast
                }
            };
            fields.push(field);
        }
        let right_parenthesis_span = parser.get_span(
            Token::Bracket(Opening::Close, Bracket::Parenthesis),
            &[
                ExpectedToken::Separator(Separator::Comma),
                ExpectedToken::Bracket(Opening::Close, Bracket::Parenthesis),
            ],
        )?;
        let syntax = match request {
            FieldTypeRequest::Either | FieldTypeRequest::Named => {
                ParenthesisSyntax::NamedFields(fields)
            }
            FieldTypeRequest::Unnamed => ParenthesisSyntax::UnnamedFields(fields),
        };
        Ok(Self {
            syntax,
            kind,
            have_splat_or_rest: star_before,
            right_parenthesis_span,
        })
    }
}
enum FieldSyntax<'a> {
    Ident(&'a str),
    Unnamed(Ast<'a>),
    Named(Ast<'a>),
    SplatOrRest(Ast<'a>),
}
struct FieldFragment<'a> {
    syntax: FieldSyntax<'a>,
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
                            syntax: FieldSyntax::Named(Tree {
                                content: Node {
                                    span: span_from_spans(
                                        parser.src,
                                        ident.span,
                                        ast.ast.content.span,
                                    ),
                                    node: NodeType::Field,
                                },
                                children: join_trees![name_ast, ast.ast],
                            }),
                            kind: ast.kind,
                        })
                    }
                    Some(Token::Separator(Separator::Comma))
                    | Some(Token::Bracket(Opening::Close, Bracket::Parenthesis)) => Ok(Self {
                        syntax: FieldSyntax::Ident(ident.span),
                        kind,
                    }),
                    Some(_) | None if field_kind.is_unnamed() => {
                        let ast = prefix::ident(parser, ident.span, kind)?;
                        Ok(Self {
                            syntax: FieldSyntax::Unnamed(ast.ast),
                            kind: ast.kind,
                        })
                    }
                    Some(_) | None => {
                        let ident_span = ident.span;
                        Err(error_start(
                            &ident_span[ident_span.len()..],
                            ErrorType::NoExpectation(&[ExpectedToken::Operator(Operator::Equal)]),
                        ))
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
                    syntax: FieldSyntax::SplatOrRest(Tree {
                        content: Node {
                            span: span_from_spans(parser.src, star_span, ast.ast.content.span),
                            node: NodeType::SplatOrRest,
                        },
                        children: join_trees![ast.ast],
                    }),
                    kind: ast.kind,
                })
            }
            Some(_) | None if field_kind.is_unnamed() => {
                let ast = parser.parse_expr(0)?;
                Ok(Self {
                    syntax: FieldSyntax::Unnamed(ast),
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
    pub fn is_unnamed(self) -> bool {
        matches!(self, Self::Unnamed | Self::Either)
    }
}
pub(super) fn field_shortcut(ident: &str) -> Ast {
    let name_ast = Tree::new(Node {
        span: ident,
        node: NodeType::Name,
    });
    let ident_ast = Tree::new(Node {
        span: ident,
        node: NodeType::Ident,
    });
    Tree {
        content: Node {
            span: ident,
            node: NodeType::Field,
        },
        children: join_trees![name_ast, ident_ast],
    }
}
