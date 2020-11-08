use crate::lexer::Bracket;
use crate::lexer::Opening;
use crate::lexer::Operator;
use crate::lexer::Token;
use crate::parser::error::ErrorType;
use crate::parser::node_type::NodeType;
use util::parser::Parse;
use util::parser::Parser;
use util::span::Span;
use util::tree_vec::Tree;

mod error;
mod infix;
mod node_type;
mod prefix;

#[derive(Clone, Copy)]
struct Node<'a> {
    span: Span<'a>,
    node: NodeType,
}
#[derive(Clone, Copy)]
struct SpanToken<'a> {
    span: Span<'a>,
    token: Token<'a>,
}
#[derive(Clone, Copy)]
struct Error<'a> {
    span: Span<'a>,
    error: ErrorType,
}
type ParseResult<'a> = Result<Tree<Node<'a>>, Vec<Error<'a>>>;
impl<'a> Parse for SpanToken<'a> {
    type Node = ParseResult<'a>;
    fn prefix_parse(tokens: &mut Parser<impl Iterator<Item = Self>>) -> Self::Node {
        match tokens.next() {
            Some(prefix) => match prefix.token {
                Token::Keyword(keyword) => prefix::keyword(prefix.span, keyword, tokens),
                Token::Operator(operator) => prefix::operator(prefix.span, operator, tokens),
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
                span: Span::eof(),
                error: ErrorType::NoExpr,
            }]),
        }
    }
    fn infix_parse(
        &self,
        left_node: Self::Node,
        tokens: &mut Parser<impl Iterator<Item = Self>>,
    ) -> Self::Node {
        match self.token {
            Token::Operator(operator) => infix::operator(left_node, operator, tokens),
            Token::Bracket(Opening::Open, bracket) => todo!(),
            _ => unreachable!(),
        }
    }
    fn infix_precedence(&self) -> Option<u32> {
        Some(match self.token {
            Token::Bracket(Opening::Open, Bracket::Bracket) => 100,
            Token::Bracket(Opening::Open, Bracket::Parenthesis) => 100,
            Token::Operator(operator) => match operator {
                Operator::Dot => 100,
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
