use crate::lexer::Bracket;
use crate::lexer::Keyword;
use crate::lexer::Opening;
use crate::lexer::Operator;
use crate::lexer::Token;
use crate::parser::ast::Ast;
use crate::parser::ast::AstType;
use crate::parser::ast::KindedAst;
use crate::parser::ast::Node;
use crate::parser::error::Error;
use crate::parser::error::ErrorType;
use crate::parser::error::ExpectedToken;
use crate::parser::node_type::NodeType;
use crate::parser::parenthesis::ParenthesisFragment;
use crate::parser::parenthesis::ParenthesisSyntax;
use crate::parser::string::parse_content;
use bracket::BracketFragment;
use bracket::BracketSyntax;
use std::iter::FusedIterator;
use std::iter::Map;
use std::iter::Peekable;
use util::iter::PeekableIterator;
use util::join_trees;
use util::lexer::LexFilter;
use util::lexer::SpanFilterIter;
use util::parser::ParserIter;
use util::span::span_from_spans;
use util::tree_vec::Tree;
use util::tree_vec::TreeVec;

mod ast;
mod bracket;
mod error;
mod float;
mod infix;
mod integer;
mod node_type;
mod parenthesis;
mod prefix;
mod string;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct SpanToken<'a> {
    span: &'a str,
    token: Token<'a>,
}
type ParserResult<'a, T> = Result<T, Vec<Error<'a>>>;
fn error_start(span: &str, error: ErrorType) -> Vec<Error> {
    vec![Error { span, error }]
}
type AstResult<'a> = ParserResult<'a, Ast<'a>>;
type KindedAstResult<'a> = ParserResult<'a, KindedAst<'a>>;
type RawParserMapper = for<'a> fn((&'a str, Token<'a>)) -> SpanToken<'a>;
struct Parser<'a> {
    src: &'a str,
    iter: Peekable<Map<SpanFilterIter<'a, Token<'a>>, RawParserMapper>>,
}
impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        // rustc 1.47.0 doesn't seem to infer this to fn pointer
        let fun: RawParserMapper = |(span, token)| SpanToken { span, token };
        Self {
            src,
            iter: Token::lex_span(src).map(fun).peekable(),
        }
    }
}
impl<'a> Iterator for Parser<'a> {
    type Item = SpanToken<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
impl<'a> PeekableIterator for Parser<'a> {
    fn peek(&mut self) -> Option<&Self::Item> {
        self.iter.peek()
    }
}
impl<'a> FusedIterator for Parser<'a> {}
impl<'a> ParserIter for Parser<'a> {
    type Ast = KindedAstResult<'a>;
    type Kind = AstType;
    fn prefix_parse(&mut self, kind: &Self::Kind) -> Self::Ast {
        let src = self.src;
        let peeked = self
            .peek()
            .ok_or_else(|| error_start(&src[src.len()..], ErrorType::no_expected_kind(*kind)))?;
        if Parser::valid_prefix(peeked.token, *kind) {
            return Err(error_start(
                &peeked.span[..0],
                ErrorType::no_expected_kind(*kind),
            ));
        }
        let prefix = self.next().unwrap();
        match prefix.token {
            Token::Keyword(keyword) => {
                prefix::keyword(self, prefix.span, keyword).map(|ast| KindedAst {
                    ast,
                    kind: AstType::Expr,
                })
            }
            Token::Operator(operator) => {
                prefix::operator(self, prefix.span, operator).map(|ast| KindedAst {
                    ast,
                    kind: AstType::Expr,
                })
            }
            Token::Int(radix, num) => {
                match integer::parse_u64(radix.as_int() as u64, num.as_bytes()) {
                    Some(num) => Ok(KindedAst {
                        kind: AstType::Expr,
                        ast: Tree::new(Node {
                            span: prefix.span,
                            node: NodeType::UInt(num),
                        }),
                    }),
                    None => Err(error_start(prefix.span, ErrorType::IntegerOverflow)),
                }
            }
            Token::Float(num) => match float::parse_float(num) {
                Some(num) => Ok(KindedAst {
                    kind: AstType::Expr,
                    ast: Tree::new(Node {
                        span: prefix.span,
                        node: NodeType::Float(num),
                    }),
                }),
                None => Err(error_start(prefix.span, ErrorType::ExpOverflow)),
            },
            Token::Bracket(Opening::Open, Bracket::Parenthesis) if kind.is_expr() => {
                let fragment = ParenthesisFragment::parse_rest(self, AstType::ExprOrUnpack, false)?;
                if fragment.kind.is_unpack()
                    && matches!(
                        self.peek_token(),
                        Some(Token::Operator(Operator::RightThickArrow)),
                    )
                {
                    let params = match fragment.syntax {
                        ParenthesisSyntax::Empty => TreeVec::new(),
                        ParenthesisSyntax::SingleIdent(ident) => {
                            join_trees![parenthesis::field_shortcut(ident)]
                        }
                        ParenthesisSyntax::Single(ast) if kind.is_unpack() => {
                            return Ok(KindedAst {
                                ast,
                                kind: AstType::Unpack,
                            });
                        }
                        ParenthesisSyntax::Single(ast) => {
                            return Err(error_start(ast.content.span, ErrorType::NotNamed))
                        }
                        ParenthesisSyntax::NamedFields(fields) => fields,
                        ParenthesisSyntax::UnnamedFields(_) => unreachable!(),
                    };
                    let param = Tree {
                        content: Node {
                            span: span_from_spans(
                                self.src,
                                prefix.span,
                                fragment.right_parenthesis_span,
                            ),
                            node: NodeType::Struct,
                        },
                        children: params,
                    };
                    self.next();
                    let body = self.parse_expr(0)?;
                    Ok(KindedAst {
                        kind: AstType::Expr,
                        ast: Tree {
                            content: Node {
                                span: span_from_spans(self.src, prefix.span, body.content.span),
                                node: NodeType::Fun,
                            },
                            children: join_trees![param, body],
                        },
                    })
                } else if fragment.kind.is_expr() {
                    let ast = fragment.into_kinded_ast(self, prefix.span);
                    let kind = if !kind.is_unpack() {
                        AstType::Expr
                    } else {
                        debug_assert_eq!(kind, AstType::ExprOrUnpack);
                        debug_assert_eq!(ast.kind, AstType::ExprOrUnpack);
                        ast.kind
                    };
                    Ok(KindedAst { kind, ast: ast.ast })
                } else {
                    let right_parenthesis_span = fragment.right_parenthesis_span;
                    Err(error_start(
                        &right_parenthesis_span[right_parenthesis_span.len()..],
                        ErrorType::NoExpectation(&[ExpectedToken::Operator(
                            Operator::RightThickArrow,
                        )]),
                    ))
                }
            }
            Token::Bracket(Opening::Open, Bracket::Parenthesis) => {
                let ast = ParenthesisFragment::parse_rest(self, AstType::Unpack, false)?
                    .into_kinded_ast(self, prefix.span);
                Ok(ast)
            }
            Token::Bracket(Opening::Open, Bracket::Bracket) => {
                let fragment = BracketFragment::parse_rest(self, *kind, false)?;
                let kind = fragment.kind;
                let (node, children) = match fragment.syntax {
                    BracketSyntax::Empty => (NodeType::Array, TreeVec::new()),
                    BracketSyntax::Single(expr) => (NodeType::Array, join_trees![expr]),
                    BracketSyntax::Multiple(elements) => (NodeType::Array, elements),
                    BracketSyntax::Range(left, range_type, right) => {
                        let children = left.into_iter().chain(right.into_iter()).collect();
                        (NodeType::ArrayRange(range_type), children)
                    }
                };
                Ok(KindedAst {
                    kind,
                    ast: Tree {
                        content: Node {
                            span: span_from_spans(
                                self.src,
                                prefix.span,
                                fragment.right_bracket_span,
                            ),
                            node,
                        },
                        children,
                    },
                })
            }
            Token::Bracket(Opening::Open, Bracket::Brace) => parse_block_rest(self, prefix.span)
                .map(|ast| KindedAst {
                    ast,
                    kind: AstType::Expr,
                }),
            Token::Str(content) => Ok(KindedAst {
                kind: AstType::Expr,
                ast: Tree {
                    content: Node {
                        span: prefix.span,
                        node: NodeType::Str,
                    },
                    children: parse_content(content)?,
                },
            }),
            Token::Char(content) => {
                let children = parse_content(content)?;
                if children.total() == 1 {
                    Ok(KindedAst {
                        kind: AstType::Expr,
                        ast: Tree {
                            content: Node {
                                span: prefix.span,
                                node: NodeType::Char,
                            },
                            children,
                        },
                    })
                } else {
                    Err(error_start(prefix.span, ErrorType::NonSingleChar))
                }
            }
            Token::Underscore => Ok(KindedAst {
                kind: AstType::Unpack,
                ast: Tree::new(Node {
                    span: prefix.span,
                    node: NodeType::Ignore,
                }),
            }),
            Token::Ident => prefix::ident(self, prefix.span, *kind),
            Token::UnterminatedQuote => Err(error_start(prefix.span, ErrorType::UnterminatedQuote)),
            Token::InvalidNumber => Err(error_start(prefix.span, ErrorType::InvalidNumber)),
            _ => unreachable!(),
        }
    }
    fn infix_parse(
        &mut self,
        left_node: Self::Ast,
        infix: Self::Item,
        kind: &Self::Kind,
    ) -> Self::Ast {
        assert!(kind.is_expr());
        let left = left_node.map(|ast| ast.ast);
        let ast = match infix.token {
            Token::Operator(operator) => infix::operator(self, left, infix.span, operator)?,
            Token::Bracket(Opening::Open, Bracket::Parenthesis) => {
                infix::call(self, left, infix.span)?
            }
            Token::Bracket(Opening::Open, Bracket::Bracket) => {
                infix::index_or_slice(self, left, false)?
            }
            _ => panic!("expected infix token, found {:?}", infix.token),
        };
        Ok(KindedAst {
            ast,
            kind: AstType::Expr,
        })
    }
    fn infix_precedence(infix: &Self::Item, kind: &Self::Kind) -> Option<u32> {
        if kind.is_expr() {
            Some(match infix.token {
                Token::Bracket(Opening::Open, Bracket::Bracket) => 100,
                Token::Bracket(Opening::Open, Bracket::Parenthesis) => 100,
                Token::Operator(operator) => match operator {
                    Operator::Dot => 100,
                    Operator::Question => 100,
                    Operator::Star => 80,
                    Operator::Slash => 80,
                    Operator::DoubleSlash => 80,
                    Operator::Percent => 80,
                    Operator::Plus => 70,
                    Operator::Minus => 70,
                    Operator::DoublePlus => 70,
                    Operator::DoubleEqual => 60,
                    Operator::NotEqual => 60,
                    Operator::Less => 60,
                    Operator::LessEqual => 60,
                    Operator::Greater => 60,
                    Operator::GreaterEqual => 60,
                    Operator::Amp => 50,
                    Operator::DoubleAmp => 50,
                    Operator::Pipe => 40,
                    Operator::DoublePipe => 40,
                    Operator::DoubleQuestion => 30,
                    Operator::LeftArrow => 20,
                    _ => return None,
                },
                _ => return None,
            })
        } else {
            None
        }
    }
}
impl<'a> Parser<'a> {
    fn peek_token(&mut self) -> Option<Token> {
        self.peek().map(|token| token.token)
    }
    fn get_span(
        &mut self,
        expectation: Token,
        more_expectation: &'static [ExpectedToken],
    ) -> ParserResult<'a, &'a str> {
        let token = self.peek();
        match token.map(|token| token.token) {
            Some(token) if token == expectation => Ok(self.next().unwrap().span),
            _ => {
                let span = match token {
                    Some(token) => &token.span[..0],
                    None => &self.src[self.src.len()..],
                };
                Err(error_start(
                    span,
                    ErrorType::NoExpectation(more_expectation),
                ))
            }
        }
    }
    fn valid_prefix(token: Token, kind: AstType) -> bool {
        match token {
            Token::Whitespace | Token::Comment => false,
            Token::Int(_, _) => kind.is_expr(),
            Token::Float(_) => kind.is_expr(),
            Token::Str(_) => kind.is_expr(),
            Token::Char(_) => kind.is_expr(),
            Token::Keyword(keyword) => {
                kind.is_expr()
                    && match keyword {
                        Keyword::True => true,
                        Keyword::False => true,
                        Keyword::Null => true,
                        Keyword::Clone => true,
                        Keyword::If => true,
                        Keyword::Else => false,
                        Keyword::For => true,
                        Keyword::In => false,
                        Keyword::Loop => true,
                        Keyword::While => true,
                        Keyword::Break => true,
                        Keyword::Continue => true,
                        Keyword::Return => true,
                    }
            }
            Token::Underscore => kind.is_unpack(),
            Token::Ident => true,
            Token::Separator(_) => false,
            Token::Bracket(Opening::Open, _) => true,
            Token::Bracket(Opening::Close, _) => false,
            Token::Operator(operator) => {
                kind.is_expr()
                    && matches!(
                        operator,
                        Operator::Plus
                            | Operator::Minus
                            | Operator::Bang
                            | Operator::Amp
                            | Operator::DoubleAmp
                            | Operator::RightThickArrow
                    )
            }
            Token::InvalidNumber => kind.is_expr(),
            Token::UnterminatedQuote => kind.is_expr(),
            Token::Unknown => false,
        }
    }
    fn parse_optional(
        &mut self,
        precedence: u32,
        kind: AstType,
    ) -> ParserResult<'a, Option<KindedAst<'a>>> {
        let peeked = match self.peek_token() {
            Some(token) => token,
            None => return Ok(None),
        };
        if Self::valid_prefix(peeked, kind) {
            self.parse(precedence, &AstType::Expr).map(Some)
        } else {
            Ok(None)
        }
    }
    fn parse_expr(&mut self, precedence: u32) -> AstResult<'a> {
        self.parse(precedence, &AstType::Expr).map(|ast| ast.ast)
    }
    fn parse_optional_expr(&mut self, precedence: u32) -> ParserResult<'a, Option<Ast<'a>>> {
        let peeked = match self.peek_token() {
            Some(token) => token,
            None => return Ok(None),
        };
        if Self::valid_prefix(peeked, AstType::Expr) {
            self.parse(precedence, &AstType::Expr)
                .map(|ast| Some(ast.ast))
        } else {
            Ok(None)
        }
    }
    fn parse_unpack(&mut self, precedence: u32) -> AstResult<'a> {
        self.parse(precedence, &AstType::Unpack).map(|ast| ast.ast)
    }
}
fn parse_block_rest<'a>(parser: &mut Parser<'a>, left_bracket_span: &'a str) -> AstResult<'a> {
    todo!()
}
fn parse_block<'a>(parser: &mut Parser<'a>) -> AstResult<'a> {
    let err_span = if let Some(token) = parser.peek() {
        if token.token != Token::Bracket(Opening::Open, Bracket::Brace) {
            let span = token.span;
            Some(&span[..0])
        } else {
            None
        }
    } else {
        let src = parser.src;
        Some(&src[src.len()..])
    };
    if let Some(span) = err_span {
        return Err(error_start(
            span,
            ErrorType::NoExpectation(&[ExpectedToken::Bracket(Opening::Open, Bracket::Brace)]),
        ));
    }
    let bracket = parser.next().unwrap();
    parse_block_rest(parser, bracket.span)
}
