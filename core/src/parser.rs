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
        if let Token::Operator(operator) = token {
            Some(match operator {
                Operator::Star | Operator::Slash | Operator::DoubleSlash | Operator::Percent => 80,
                Operator::Plus | Operator::Minus | Operator::PlusPlus => 70,
                Operator::DoubleEqual
                | Operator::NotEqual
                | Operator::Less
                | Operator::LessEqual
                | Operator::Greater
                | Operator::GreaterEqual => 60,
                Operator::Amp | Operator::DoubleAmp => 50,
                Operator::Pipe | Operator::DoublePipe => 40,
                Operator::DoubleQuestion => 30,
                Operator::LeftArrow => 20,
                _ => return None,
            })
        } else {
            None
        }
    }
}
