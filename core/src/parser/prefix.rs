use crate::lexer::Bracket;
use crate::lexer::Keyword;
use crate::lexer::Opening;
use crate::lexer::Operator;
use crate::lexer::Token;
use crate::parser::ast::Ast;
use crate::parser::ast::AstType;
use crate::parser::error_start;
use crate::parser::node_type::NodeType;
use crate::parser::node_type::Unary;
use crate::parser::parse_block;
use crate::parser::parse_block_rest;
use crate::parser::AstResult;
use crate::parser::ErrorType;
use crate::parser::KindedAst;
use crate::parser::KindedAstResult;
use crate::parser::Node;
use crate::parser::Parser;
use crate::parser::TokenKind;
use util::join_trees;
use util::span::span_from_spans;
use util::tree_vec::Tree;

pub(super) fn operator<'a>(
    parser: &mut Parser<'a>,
    span: &'a str,
    operator: Operator,
) -> AstResult<'a> {
    match operator {
        Operator::Plus | Operator::Minus | Operator::Bang | Operator::Amp => {
            unary_operator(parser, span, operator)
        }
        Operator::DoubleAmp => double_ref(parser, span),
        Operator::RightThickArrow => nil_function(parser, span),
        operator => panic!("expected prefix operator, found: {:?}", operator),
    }
}
pub(super) fn keyword<'a>(
    parser: &mut Parser<'a>,
    span: &'a str,
    keyword: Keyword,
) -> AstResult<'a> {
    match keyword {
        Keyword::True | Keyword::False | Keyword::Null => Ok(Tree::new(Node {
            span,
            node: keyword_literal(keyword),
        })),
        Keyword::Clone => clone(parser, span),
        Keyword::If => parse_if(parser, span),
        Keyword::For => parse_for(parser, span),
        Keyword::Loop => parse_loop(parser, span),
        Keyword::While => parse_while(parser, span),
        Keyword::Break => parse_break(parser, span),
        Keyword::Continue => Ok(parse_continue(parser, span)),
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
fn clone<'a>(parser: &mut Parser<'a>, span: &'a str) -> AstResult<'a> {
    let operand = parser.parse_expr(90)?;
    Ok(Tree {
        content: Node {
            span: span_from_spans(parser.src, span, operand.content.span),
            node: NodeType::Unary(Unary::Clone),
        },
        children: join_trees![operand],
    })
}
fn parse_continue<'a>(parser: &mut Parser<'a>, span: &'a str) -> Ast<'a> {
    let label = optional_label(parser);
    Tree {
        content: Node {
            span: match &label {
                Some(label) => span_from_spans(parser.src, span, label.content.span),
                None => span,
            },
            node: NodeType::Continue,
        },
        children: label.into_iter().collect(),
    }
}
fn parse_break<'a>(parser: &mut Parser<'a>, span: &'a str) -> AstResult<'a> {
    let label = optional_label(parser);
    let expr = if let Some(Token::Operator(Operator::Equal)) = parser.peek_token() {
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
fn parse_return<'a>(parser: &mut Parser<'a>, span: &'a str) -> AstResult<'a> {
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
fn unary_operator<'a>(parser: &mut Parser<'a>, span: &'a str, operator: Operator) -> AstResult<'a> {
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
fn double_ref<'a>(parser: &mut Parser<'a>, span: &'a str) -> AstResult<'a> {
    let operand = parser.parse_expr(90)?;
    debug_assert!(span == "&&");
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
fn optional_label<'a>(parser: &mut Parser<'a>) -> Option<Ast<'a>> {
    match parser.peek_token()? {
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
fn parse_loop<'a>(parser: &mut Parser<'a>, span: &'a str) -> AstResult<'a> {
    let block = parse_block(parser)?;
    Ok(Tree {
        content: Node {
            span: span_from_spans(parser.src, span, block.content.span),
            node: NodeType::Loop,
        },
        children: block.children,
    })
}
fn parse_while<'a>(parser: &mut Parser<'a>, span: &'a str) -> AstResult<'a> {
    let condition = parser.parse_expr(0)?;
    let block = parse_block(parser)?;
    Ok(Tree {
        content: Node {
            span: span_from_spans(parser.src, span, block.content.span),
            node: NodeType::While,
        },
        children: join_trees![condition, block],
    })
}
fn parse_if<'a>(parser: &mut Parser<'a>, span: &'a str) -> AstResult<'a> {
    let condition = parser.parse_expr(0)?;
    let block = parse_block(parser)?;
    let block_span = block.content.span;
    let mut children = join_trees![condition, block];
    let end_span;
    if let Some(Token::Keyword(Keyword::Else)) = parser.peek_token() {
        let else_span = parser.next().unwrap().span;
        let else_tree = parse_else(parser, else_span)?;
        end_span = else_tree.content.span;
        children.push(else_tree);
    } else {
        end_span = block_span;
    }
    Ok(Tree {
        content: Node {
            span: span_from_spans(parser.src, span, end_span),
            node: NodeType::If,
        },
        children,
    })
}
fn parse_else<'a>(parser: &mut Parser<'a>, span: &'a str) -> AstResult<'a> {
    match parser.peek_token() {
        Some(Token::Bracket(Opening::Open, Bracket::Brace)) => {
            let token_span = parser.next().unwrap().span;
            parse_block_rest(parser, token_span)
        }
        Some(Token::Keyword(Keyword::If)) => {
            let token_span = parser.next().unwrap().span;
            parse_if(parser, token_span)
        }
        Some(_) | None => Err(error_start(
            &span[span.len()..],
            ErrorType::NoExpectation(&[
                TokenKind::Bracket(Opening::Open, Bracket::Brace),
                TokenKind::Keyword(Keyword::If),
            ]),
        )),
    }
}
fn parse_for<'a>(parser: &mut Parser<'a>, span: &'a str) -> AstResult<'a> {
    let iter_unpack = parser.parse_unpack(0)?;
    if let Some(Token::Keyword(Keyword::In)) = parser.peek_token() {
        parser.next();
    } else {
        let span = iter_unpack.content.span;
        return Err(error_start(
            &span[span.len()..],
            ErrorType::NoExpectation(&[TokenKind::Keyword(Keyword::In)]),
        ));
    }
    let iter_val = parser.parse_expr(0)?;
    let block = parse_block(parser)?;
    Ok(Tree {
        content: Node {
            span: span_from_spans(parser.src, span, block.content.span),
            node: NodeType::For,
        },
        children: join_trees![iter_unpack, iter_val, block],
    })
}
fn nil_function<'a>(parser: &mut Parser<'a>, span: &'a str) -> AstResult<'a> {
    let parameter = Tree::new(Node {
        span: &span[..0],
        node: NodeType::Struct,
    });
    let body = parser.parse_expr(0)?;
    Ok(Tree {
        content: Node {
            span: span_from_spans(parser.src, span, body.content.span),
            node: NodeType::Fun,
        },
        children: join_trees![parameter, body],
    })
}
pub(super) fn ident<'a>(
    parser: &mut Parser<'a>,
    span: &'a str,
    kind: AstType,
) -> KindedAstResult<'a> {
    if Some(Token::Operator(Operator::RightThickArrow)) == parser.peek_token() && kind.is_expr() {
        parser.next();
        let name = Tree::new(Node {
            span,
            node: NodeType::Name,
        });
        let ident = Tree::new(Node {
            span,
            node: NodeType::Ident,
        });
        let field = Tree {
            content: Node {
                span,
                node: NodeType::NamedField,
            },
            children: join_trees![name, ident],
        };
        let parameter = Tree {
            content: Node {
                span,
                node: NodeType::Struct,
            },
            children: join_trees![field],
        };
        let body = parser.parse_expr(0)?;
        Ok(KindedAst {
            kind: AstType::Expr,
            ast: Tree {
                content: Node {
                    span: span_from_spans(parser.src, span, body.content.span),
                    node: NodeType::Fun,
                },
                children: join_trees![parameter, body],
            },
        })
    } else {
        Ok(KindedAst {
            kind: AstType::ExprOrUnpack,
            ast: Tree::new(Node {
                span,
                node: NodeType::Ident,
            }),
        })
    }
}
