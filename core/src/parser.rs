use crate::lexer::Bracket;
use crate::lexer::Keyword;
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
impl<'a> PeekableIter for Parser<'a> {
    fn peek(&mut self) -> Option<&Self::Item> {
        self.iter.peek()
    }
}
impl<'a> ParserIter for Parser<'a> {
    type Node = ParseResult<'a>;
    fn prefix_parse(&mut self) -> Self::Node {
        let src = self.src;
        let peeked = match self.peek() {
            Some(token) => token,
            None => {
                return Err(vec![Error {
                    span: unsafe { src.get_unchecked(src.len()..) },
                    error: ErrorType::NoExpr,
                }]);
            }
        };
        if Parser::valid_prefix(peeked.token) {
            return Err(vec![Error {
                span: unsafe { peeked.span.get_unchecked(..0) },
                error: ErrorType::NoExpr,
            }]);
        }
        let prefix = self.next().unwrap();
        match prefix.token {
            Token::Keyword(keyword) => prefix::keyword(self, prefix.span, keyword),
            Token::Operator(operator) => prefix::operator(self, prefix.span, operator),
            Token::UnterminatedQuote => Err(vec![Error {
                span: prefix.span,
                error: ErrorType::UnterminatedQuote,
            }]),
            Token::Bracket(Opening::Open, bracket) => todo!(),
            Token::Str(content) => todo!(),
            Token::Char(content) => todo!(),
            Token::Ident => todo!(),
            _ => unreachable!(),
        }
    }
    fn infix_parse(&mut self, left_node: Self::Node, infix: Self::Item) -> Self::Node {
        match infix.token {
            Token::Operator(operator) => infix::operator(self, left_node, infix.span, operator),
            Token::Bracket(Opening::Open, bracket) => todo!(),
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
    fn valid_prefix(token: Token) -> bool {
        match token {
            Token::Whitespace | Token::Comment => false,
            Token::Num => true,
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
                Keyword::Loop => false,
                Keyword::While => true,
                Keyword::Break => true,
                Keyword::Continue => true,
                Keyword::Return => true,
            },
            Token::Ident => true,
            Token::Separator(_) => false,
            Token::Bracket(Opening::Open, _) => true,
            Token::Bracket(Opening::Close, _) => false,
            Token::Operator(operator) => match operator {
                Operator::Plus => true,
                Operator::Minus => true,
                Operator::Bang => true,
                Operator::Amp => true,
                Operator::DoubleAmp => true,
                Operator::RightThickArrow => true,
                _ => false,
            },
            Token::UnterminatedQuote => true,
            Token::Unknown => false,
        }
    }
    fn parse_expr(&mut self, precedence: u32) -> ParseResult<'a> {
        self.partial_parse(precedence).and_then(assert_expr)
    }
}
fn assert_expr(node: Tree<Node>) -> ParseResult {
    if node.content.node.expr() {
        Ok(node)
    } else {
        Err(vec![Error {
            span: node.content.span,
            error: ErrorType::NonExpr,
        }])
    }
}
