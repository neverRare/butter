use crate::lexer::Bracket;
use crate::lexer::Opening;
use crate::lexer::Operator;
use crate::lexer::Token;
use crate::parser::error::ErrorType;
use crate::parser::node_type::NodeType;
use std::iter::Map;
use std::iter::Peekable;
use util::iter::PeekableIter;
use util::lexer::LexFilter;
use util::lexer::SpanFilterIter;
use util::parser::ParserIter;
use util::tree_vec::Tree;

mod error;
mod infix;
mod node_type;
mod prefix;

#[derive(Clone, Copy)]
struct Node<'a> {
    span: &'a str,
    node: NodeType,
}
#[derive(Clone, Copy)]
struct SpanToken<'a> {
    span: &'a str,
    token: Token<'a>,
}
#[derive(Clone, Copy)]
struct Error<'a> {
    span: &'a str,
    error: ErrorType,
}
type ParseResult<'a> = Result<Tree<Node<'a>>, Vec<Error<'a>>>;
struct Parser<'a> {
    src: &'a str,
    iter: Peekable<Map<SpanFilterIter<'a, Token<'a>>, fn((&'a str, Token<'a>)) -> SpanToken<'a>>>,
}
impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src,
            iter: Token::lex_span(src)
                .map(|(span, token)| SpanToken { span, token })
                .peekable(),
        }
    }
}
impl<'a> Iterator for Parser<'a> {
    type Item = SpanToken<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
impl<'a> PeekableIter for Parser<'a> {
    fn peek(&mut self) -> Option<&Self::Item> {
        self.iter.peek()
    }
}
impl<'a> ParserIter for Parser<'a> {
    type Node = ParseResult<'a>;
    fn prefix_parse(&mut self) -> Self::Node {
        match self.next() {
            Some(prefix) => match prefix.token {
                Token::Keyword(keyword) => prefix::keyword(prefix.span, keyword, self),
                Token::Operator(operator) => prefix::operator(prefix.span, operator, self),
                Token::UnterminatedQuote => Err(vec![Error {
                    span: prefix.span,
                    error: ErrorType::UnterminatedQuote,
                }]),
                Token::Unknown => Err(vec![Error {
                    span: prefix.span,
                    error: ErrorType::UnknownToken,
                }]),
                Token::Whitespace | Token::Comment => {
                    unreachable!("unexpected insignificant token")
                }
                _ => todo!(),
            },
            None => Err(vec![Error {
                span: self.eof(),
                error: ErrorType::SuddenEof,
            }]),
        }
    }
    fn infix_parse(&mut self, left_node: Self::Node, infix: Self::Item) -> Self::Node {
        match infix.token {
            Token::Operator(operator) => infix::operator(left_node, infix.span, operator, self),
            Token::Bracket(Opening::Open, bracket) => todo!(),
            _ => unreachable!(),
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
    fn parse_expr(&mut self, precedence: u32) -> ParseResult<'a> {
        self.partial_parse(precedence).and_then(assert_expr)
    }
    fn span_from_spans(&self, left: &'a str, right: &'a str) -> &'a str {
        let src = self.src;
        let src_pos = src.as_ptr() as usize;
        let left = src_pos - left.as_ptr() as usize;
        let right = src_pos - right.as_ptr() as usize + right.len();
        &src[left..right]
    }
    fn eof(&self) -> &'a str {
        &self.src[self.src.len()..]
    }
}
fn assert_expr(node: Tree<Node>) -> ParseResult {
    if node.content.node.expr() {
        Ok(node)
    } else {
        Err(vec![Error {
            span: node.content.span,
            error: ErrorType::NonExprOperand,
        }])
    }
}
