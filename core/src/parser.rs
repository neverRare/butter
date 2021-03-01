use crate::lexer::Bracket;
use crate::lexer::Keyword;
use crate::lexer::Opening;
use crate::lexer::Operator;
use crate::lexer::Token;
use crate::parser::error::ErrorType;
use crate::parser::error::TokenKind;
use crate::parser::node_type::NodeType;
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

mod bracket;
mod error;
mod float;
mod infix;
mod integer;
mod node_type;
mod prefix;
mod string;

#[derive(Clone, Copy, PartialEq, Debug)]
struct Node<'a> {
    span: &'a str,
    node: NodeType,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct SpanToken<'a> {
    span: &'a str,
    token: Token<'a>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Error<'a> {
    span: &'a str,
    error: ErrorType,
}
type ParserResult<'a, T> = Result<T, Vec<Error<'a>>>;
fn error_start(span: &str, error: ErrorType) -> Vec<Error> {
    vec![Error { span, error }]
}
type ParseResult<'a> = ParserResult<'a, Tree<Node<'a>>>;
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
    type Node = ParseResult<'a>;
    fn prefix_parse(&mut self) -> Self::Node {
        let src = self.src;
        let peeked = self
            .peek()
            .ok_or_else(|| error_start(&src[src.len()..], ErrorType::NoExpr))?;
        if Parser::valid_prefix(peeked.token) {
            return Err(error_start(&peeked.span[..0], ErrorType::NoExpr));
        }
        let prefix = self.next().unwrap();
        match prefix.token {
            Token::Keyword(keyword) => prefix::keyword(self, prefix.span, keyword),
            Token::Operator(operator) => prefix::operator(self, prefix.span, operator),
            Token::Int(radix, num) => {
                match integer::parse_u64(radix.as_int() as u64, num.as_bytes()) {
                    Some(num) => Ok(Tree::new(Node {
                        span: prefix.span,
                        node: NodeType::UInt(num),
                    })),
                    None => Err(error_start(prefix.span, ErrorType::IntegerOverflow)),
                }
            }
            Token::Float(num) => match float::parse_float(num) {
                Some(num) => Ok(Tree::new(Node {
                    span: prefix.span,
                    node: NodeType::Float(num),
                })),
                None => Err(error_start(prefix.span, ErrorType::ExpOverflow)),
            },
            Token::Bracket(Opening::Open, Bracket::Parenthesis) => todo!(),
            Token::Bracket(Opening::Open, Bracket::Bracket) => {
                let fragment = BracketFragment::parse_rest(self)?;
                let (node, children) = match fragment.syntax {
                    BracketSyntax::Empty => (NodeType::Array, TreeVec::new()),
                    BracketSyntax::Single(expr) => (NodeType::Array, join_trees![expr]),
                    BracketSyntax::Multiple(elements) => (NodeType::Array, elements),
                    BracketSyntax::Range(range_type, bounds) => {
                        (NodeType::ArrayRange(range_type), bounds)
                    }
                };
                Ok(Tree {
                    content: Node {
                        span: span_from_spans(self.src, prefix.span, fragment.right_bracket_span),
                        node,
                    },
                    children,
                })
            }
            Token::Bracket(Opening::Open, Bracket::Brace) => parse_block_rest(self, prefix.span),
            Token::Str(content) => Ok(Tree {
                content: Node {
                    span: prefix.span,
                    node: NodeType::Str,
                },
                children: parse_content(content)?,
            }),
            Token::Char(content) => {
                let children = parse_content(content)?;
                if children.total() == 1 {
                    Ok(Tree {
                        content: Node {
                            span: prefix.span,
                            node: NodeType::Char,
                        },
                        children,
                    })
                } else {
                    Err(error_start(prefix.span, ErrorType::NonSingleChar))
                }
            }
            Token::Ident => todo!(),
            Token::UnterminatedQuote => Err(error_start(prefix.span, ErrorType::UnterminatedQuote)),
            Token::InvalidNumber => Err(error_start(prefix.span, ErrorType::InvalidNumber)),
            _ => unreachable!(),
        }
    }
    fn infix_parse(&mut self, left_node: Self::Node, infix: Self::Item) -> Self::Node {
        match infix.token {
            Token::Operator(operator) => infix::operator(self, left_node, infix.span, operator),
            Token::Bracket(Opening::Open, Bracket::Parenthesis) => todo!(),
            Token::Bracket(Opening::Open, Bracket::Bracket) => todo!(),
            _ => panic!("expected infix token, found {:?}", infix.token),
        }
    }
    fn infix_precedence(infix: &Self::Item) -> Option<u32> {
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
    }
}
impl<'a> Parser<'a> {
    fn peek_token(&mut self) -> Option<Token> {
        self.peek().map(|token| token.token)
    }
    fn valid_prefix(token: Token) -> bool {
        match token {
            Token::Whitespace | Token::Comment => false,
            Token::Int(_, _) => true,
            Token::Float(_) => true,
            Token::Str(_) => true,
            Token::Char(_) => true,
            Token::Keyword(keyword) => match keyword {
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
            },
            Token::Underscore => false,
            Token::Ident => true,
            Token::Separator(_) => false,
            Token::Bracket(Opening::Open, _) => true,
            Token::Bracket(Opening::Close, _) => false,
            Token::Operator(operator) => matches!(
                operator,
                Operator::Plus
                    | Operator::Minus
                    | Operator::Bang
                    | Operator::Amp
                    | Operator::DoubleAmp
                    | Operator::RightThickArrow
            ),
            Token::InvalidNumber => true,
            Token::UnterminatedQuote => true,
            Token::Unknown => false,
        }
    }
    fn parse_optional_expr(&mut self, precedence: u32) -> ParserResult<'a, Option<Tree<Node<'a>>>> {
        let peeked = match self.peek_token() {
            Some(token) => token,
            None => return Ok(None),
        };
        if Self::valid_prefix(peeked) {
            self.partial_parse(precedence).map(Some)
        } else {
            Ok(None)
        }
    }
}
fn parse_block_rest<'a>(parser: &mut Parser<'a>, left_bracket_span: &'a str) -> ParseResult<'a> {
    todo!()
}
fn parse_block<'a>(parser: &mut Parser<'a>) -> ParseResult<'a> {
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
            ErrorType::NoExpectation(&[TokenKind::Bracket(Opening::Open, Bracket::Brace)]),
        ));
    }
    let bracket = parser.next().unwrap();
    parse_block_rest(parser, bracket.span)
}
