use crate::expr::array::range;
use crate::expr::expr;
use crate::expr::record;
use crate::expr::Expr;
use crate::ident_keyword::ident;
use crate::lex;
use combine::attempt;
use combine::between;
use combine::choice;
use combine::error::StreamError;
use combine::many;
use combine::not_followed_by;
use combine::optional;
use combine::parser::char::char;
use combine::parser::char::string;
use combine::parser::range::recognize;
use combine::sep_end_by;
use combine::stream::StreamErrorFor;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;
use hir::expr::compound::Record;
use hir::expr::operator::Assign;
use hir::expr::operator::NamedArgCall;
use hir::expr::operator::Property;
use hir::expr::operator::Slice;
use hir::expr::operator::UnnamedArgCall;
use hir::expr::range::Range;
use hir::expr::Binary;
use hir::expr::BinaryType;
use hir::expr::Index;
use hir::expr::PlaceExpr;

pub enum PartialAst<'a, T> {
    Property(&'a str),
    Index(Expr<'a, T>),
    Slice(Range<'a, T>),
    NamedArgCall(Record<'a, T>),
    UnnamedArgCall(Box<[Expr<'a, T>]>),
    Deref,
    Len,
}
impl<'a, T> PartialAst<'a, T> {
    pub fn combine_from(self, left: Expr<'a, T>) -> Expr<'a, T> {
        match self {
            Self::Property(name) => Expr::Property(Property {
                expr: Box::new(left),
                name,
            }),
            Self::Index(index) => Expr::Index(Index {
                expr: Box::new(left),
                index: Box::new(index),
            }),
            Self::Slice(range) => Expr::Slice(Slice {
                expr: Box::new(left),
                range,
            }),
            Self::NamedArgCall(args) => Expr::NamedArgCall(NamedArgCall {
                expr: Box::new(left),
                args,
            }),
            Self::UnnamedArgCall(args) => Expr::UnnamedArgCall(UnnamedArgCall {
                expr: Box::new(left),
                args,
            }),
            Self::Deref => Expr::Deref(Box::new(left)),
            Self::Len => Expr::Len(Box::new(left)),
        }
    }
}
fn infix_6<'a, I, T>() -> impl Parser<I, Output = PartialAst<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    // TODO: disallow variable but allow (variable) possibly with lookahead
    let nameless_arg = || expr(0);
    let nameless_args = || {
        between(
            lex(char('(')),
            lex(char(')')),
            sep_end_by(nameless_arg(), lex(char(','))),
        )
        .map(Vec::into)
    };
    let property_or_len = || {
        lex(attempt(
            char('.')
                .skip(not_followed_by(char('<')))
                .skip(not_followed_by(char('.'))),
        ))
        .with(lex(ident()))
        .map(|prop| {
            if prop == "len" {
                PartialAst::Len
            } else {
                PartialAst::Property(prop)
            }
        })
    };
    let index = || between(lex(char('[')), lex(char(']')), expr(0)).map(PartialAst::Index);
    choice((
        attempt(record()).map(PartialAst::NamedArgCall),
        nameless_args().map(PartialAst::UnnamedArgCall),
        property_or_len(),
        attempt(index()),
        range().map(PartialAst::Slice),
        lex(char('^')).map(|_| PartialAst::Deref),
    ))
}
pub fn expr_6<'a, I, T>() -> impl Parser<I, Output = Expr<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    (expr(7), many(infix_6())).map(|(prefix, infixes)| {
        let infixes: Vec<_> = infixes;
        let mut expr = prefix;
        for infix in infixes {
            expr = infix.combine_from(expr);
        }
        expr
    })
}
pub fn expr_0<'a, I, T>() -> impl Parser<I, Output = Expr<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    (expr(1), optional(lex(attempt(string("<-"))).with(expr(0)))).and_then(|(place, expr)| {
        match expr {
            Some(expr) => match PlaceExpr::from_expr(place) {
                Some(place) => Ok(Expr::Assign(Assign {
                    place: Box::new(place),
                    expr: Box::new(expr),
                })),
                None => Err(<StreamErrorFor<I>>::message_static_message(
                    "non place expression",
                )),
            },
            None => Ok(place),
        }
    })
}
pub fn precedence_of(token: &str) -> Option<u8> {
    match token {
        "." | "[" | "(" | "^" => Some(7),
        "*" | "/" | "//" | "%" => Some(6),
        "+" | "-" | "++" => Some(5),
        "==" | "!=" | "<" | ">" | "<=" | ">=" => Some(4),
        "&&" | "&" => Some(3),
        "||" | "|" => Some(2),
        "<-" => Some(1),
        _ => None,
    }
}
pub fn infix_expr_op<'a, I, T>(
    precedence: u8,
) -> impl Parser<I, Output = impl Fn(Expr<'a, T>, Expr<'a, T>) -> Expr<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let double_ops = || {
        choice([
            attempt(string("//")),
            attempt(string("==")),
            attempt(string("!=")),
            attempt(string("<=")),
            attempt(string(">=")),
            attempt(string("&&")),
            attempt(string("||")),
            attempt(string("<-")),
            attempt(string("..")),
            attempt(string(".<")),
            attempt(string(">.")),
            attempt(string("><")),
        ])
    };
    let single_ops = || {
        choice([
            char('.'),
            char('['),
            char('('),
            char('^'),
            char('*'),
            char('/'),
            char('%'),
            char('+'),
            char('-'),
            char('<'),
            char('>'),
            char('&'),
            char('|'),
        ])
    };
    lex(choice((double_ops(), recognize(single_ops()))))
        .and_then(move |token| match precedence_of(token) {
            Some(this_precedence) if this_precedence > precedence => Ok(token),
            Some(_) => Err(<StreamErrorFor<I>>::unexpected_static_message(
                "infix operator with higher precedence",
            )),
            None => Err(<StreamErrorFor<I>>::expected_static_message(
                "expression operator",
            )),
        })
        .and_then(|op| {
            let op = match op {
                "+" => BinaryType::Add,
                "-" => BinaryType::Sub,
                "*" => BinaryType::Multiply,
                "/" => BinaryType::Div,
                "//" => BinaryType::FloorDiv,
                "%" => BinaryType::Mod,
                "&" => BinaryType::And,
                "|" => BinaryType::Or,
                "||" => BinaryType::LazyOr,
                "==" => BinaryType::Equal,
                "!=" => BinaryType::NotEqual,
                "<" => BinaryType::Less,
                ">" => BinaryType::Greater,
                "<=" => BinaryType::LessEqual,
                ">=" => BinaryType::GreaterEqual,
                "++" => BinaryType::Concatenate,
                "&&" => BinaryType::LazyAnd,
                _ => {
                    return Err(<StreamErrorFor<I>>::expected_static_message(
                        "infix expression operator",
                    ))
                }
            };
            Ok(op)
        })
        .map(|op| {
            move |left, right| {
                Expr::Binary(Binary {
                    kind: op,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
        })
}
