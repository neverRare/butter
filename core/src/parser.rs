use crate::lexer::Bracket;
use crate::lexer::Opening;
use crate::lexer::Operator;
use crate::lexer::Token;
use std::iter::Peekable;
use util::lexer::LexFilter;
use util::parser::Parser;
use util::span::Span;
use util::tree_vec::Tree;
use util::tree_vec::TreeVec;

mod prefix_parselet;

#[derive(Clone, Copy)]
struct SpanToken<'a> {
    span: Span<'a>,
    token: Token<'a>,
}
#[derive(Clone, Copy)]
enum Num {
    UInt(u64),
    Float(f64),
}
#[derive(Clone, Copy)]
enum UnaryOp {
    Plus,
    Minus,
    Ref,
    Not,
    Clone,
}
#[derive(Clone, Copy)]
enum BinaryOp {
    Add,
    Sub,
    Mult,
    Div,
    FlrDiv,
    Mod,
    And,
    Or,
    LazyAnd,
    LazyOr,
    Eq,
    NotEq,
    Gt,
    Gte,
    Lt,
    Lte,
    Concat,
}
#[derive(Clone, Copy)]
enum NodeType {
    SplatOrRest,
    Label,

    CharInside(u8),

    True,
    False,
    Null,
    Ident,
    Char,
    Str,
    Num(Num),
    Path,

    Abort,
    Break,
    Continue,
    Return,

    Unary(UnaryOp),
    Binary(BinaryOp),

    Declare,
    FunDeclare,
    Assign,

    Array,
    Struct,

    Property,
    OptionalProperty,
    Index,
    OptionalIndex,

    Block(bool),
    Fun,
    If,
    Else,
    For,
    While,
    Loop,
}
#[derive(Clone, Copy)]
struct Node<'a> {
    span: Span<'a>,
    node: NodeType,
    unpack: bool,
}
impl<'a> Node<'a> {
    fn get_statements(tokens: &mut Peekable<impl Iterator<Item = SpanToken<'a>>>) -> TreeVec<Self> {
        todo!();
    }
    fn parse(src: &'a str) -> TreeVec<Self> {
        // TODO handle unparsed tokens
        Self::get_statements(
            &mut Token::lex_span(src)
                .map(|(span, token)| SpanToken {
                    span: Span::from_str(src, span),
                    token,
                })
                .peekable(),
        )
    }
}
macro_rules! prefix_parselets {
    (($prefix:expr, $tokens:expr $(,)?); $($path:path,)* => else $else:expr $(,)?) => {{
        let prefix = $prefix;
        let tokens = $tokens;
        $(
            if let Some(node) = $path(prefix, tokens) {
                node
            } else
        )* {
            $else
        }
    }};
}
impl<'a> Parser for Node<'a> {
    type Token = SpanToken<'a>;
    fn prefix_parse(tokens: &mut Peekable<impl Iterator<Item = Self::Token>>) -> Tree<Self> {
        let prefix = tokens.next().unwrap();
        prefix_parselets! {
            (prefix, tokens);
            prefix_parselet::operator,
            prefix_parselet::clone,
            prefix_parselet::keyword_literal,
            => else panic!("Prefix token remained unhandled: {:?}", prefix.token),
        }
    }
    fn infix_parse(
        left_node: Tree<Self>,
        infix: Self::Token,
        tokens: &mut Peekable<impl Iterator<Item = Self::Token>>,
    ) -> Tree<Self> {
        // TODO infix parselets here
        panic!("Infix token remained unhandled: {:?}", infix.token);
    }
    fn infix_precedence(token: &Self::Token) -> Option<u32> {
        let SpanToken { span: _, token } = token;
        Some(match token {
            Token::Bracket(Opening::Open, Bracket::Bracket) => 100,
            Token::Bracket(Opening::Open, Bracket::Paren) => 100,
            Token::Operator(operator) => match operator {
                Operator::Dot => 100,
                Operator::Star => 80,
                Operator::Slash => 80,
                Operator::DoubleSlash => 80,
                Operator::Percent => 80,
                Operator::Plus => 70,
                Operator::Minus => 70,
                Operator::PlusPlus => 70,
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