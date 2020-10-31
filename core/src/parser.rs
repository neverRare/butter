use crate::lexer::Bracket;
use crate::lexer::Opening;
use crate::lexer::Operator;
use crate::lexer::Token;
use std::iter::Peekable;
use util::parser::Parser;
use util::tree_vec::Tree;

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

    Error,
}
struct Node<'a> {
    src: &'a str,
    node: NodeType,
}
impl<'a> Parser for Node<'a> {
    type Token = (&'a str, Token<'a>);
    fn error_node() -> Self {
        Self {
            src: "",
            node: NodeType::Error,
        }
    }
    fn prefix_parse(
        prefix: Self::Token,
        tokens: &mut Peekable<impl Iterator<Item = Self::Token>>,
    ) -> Tree<Self> {
        todo!();
    }
    fn infix_parse(
        left_node: Tree<Self>,
        infix: Self::Token,
        tokens: &mut Peekable<impl Iterator<Item = Self::Token>>,
    ) -> Tree<Self> {
        todo!();
    }
    fn infix_precedence((_, token): &Self::Token) -> Option<u32> {
        Some(match token {
            Token::Bracket(Opening::Open, Bracket::Bracket) => 90,
            Token::Bracket(Opening::Open, Bracket::Paren) => 90,
            Token::Operator(operator) => match operator {
                Operator::Dot => 90,
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
