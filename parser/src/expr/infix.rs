use super::tuple::tuple;
use crate::{
    expr::{array::range, expr, record},
    ident_keyword::ident,
    lex,
};
use combine::{
    attempt, between, choice,
    error::StreamError,
    many, not_followed_by, optional,
    parser::{
        char::{char, string},
        range::recognize,
    },
    stream::StreamErrorFor,
    ParseError, Parser, RangeStream,
};
use hir::expr::{
    Arg, Assign, Binary, BinaryType, Call, Expr, FieldAccess, Index, PlaceExpr, Range, Record,
    Slice, Tuple,
};

pub(crate) enum PartialAst<'a, T> {
    Property(&'a str),
    Index(Expr<'a, T>),
    Slice(Range<'a, T>),
    UnitCall,
    SplatCall(Expr<'a, T>),
    RecordCall(Record<'a, T>),
    TupleCall(Tuple<'a, T>),
    Deref,
    Len,
}
impl<'a, T> PartialAst<'a, T> {
    pub(crate) fn combine_from(self, left: Expr<'a, T>) -> Expr<'a, T> {
        match self {
            Self::Property(name) => Expr::Place(PlaceExpr::FieldAccess(FieldAccess {
                expr: Box::new(left),
                name,
            })),
            Self::Index(index) => Expr::Place(PlaceExpr::Index(Index {
                expr: Box::new(left),
                index: Box::new(index),
            })),
            Self::Slice(range) => Expr::Place(PlaceExpr::Slice(Slice {
                expr: Box::new(left),
                range,
            })),
            Self::UnitCall => Expr::Call(Call {
                expr: Box::new(left),
                arg: Arg::Unit,
            }),
            Self::SplatCall(arg) => Expr::Call(Call {
                expr: Box::new(left),
                arg: Arg::Splat(Box::new(arg)),
            }),
            Self::RecordCall(arg) => Expr::Call(Call {
                expr: Box::new(left),
                arg: Arg::Record(arg),
            }),
            Self::TupleCall(arg) => Expr::Call(Call {
                expr: Box::new(left),
                arg: Arg::Tuple(arg),
            }),
            Self::Deref => Expr::Place(PlaceExpr::Deref(Box::new(left))),
            Self::Len => Expr::Place(PlaceExpr::Len(Box::new(left))),
        }
    }
}
fn infix_6<'a, I, T>() -> impl Parser<I, Output = PartialAst<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
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
    let index = || {
        between(lex(char('[')), lex(char(']')), expr(0))
            .map(PartialAst::Index)
            .expected("index")
    };
    let arg = || {
        choice((
            attempt((lex(char('(')), lex(char(')')))).map(|_| PartialAst::UnitCall),
            attempt(between(
                (lex(char('(')), lex(char('*'))),
                (optional(lex(char(','))), lex(char(')'))),
                expr(0),
            ))
            .map(|expr| PartialAst::SplatCall(expr)),
            attempt(tuple()).map(PartialAst::TupleCall),
            attempt(record()).map(PartialAst::RecordCall),
        ))
        .expected("argument")
    };
    choice((
        arg(),
        property_or_len(),
        attempt(index()),
        range().map(PartialAst::Slice).expected("slice"),
        lex(char('^')).map(|_| PartialAst::Deref),
    ))
}
pub(crate) fn expr_6<'a, I, T>() -> impl Parser<I, Output = Expr<'a, T>>
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
pub(crate) fn expr_0<'a, I, T>() -> impl Parser<I, Output = Expr<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    (expr(1), optional(lex(attempt(string("<-"))).with(expr(0)))).and_then(|(place, expr)| {
        match expr {
            Some(expr) => {
                if let Expr::Place(place) = place {
                    Ok(Expr::Assign(Assign {
                        place: Box::new(place),
                        expr: Box::new(expr),
                    }))
                } else {
                    Err(<StreamErrorFor<I>>::message_static_message(
                        "non place expression",
                    ))
                }
            }
            None => Ok(place),
        }
    })
}
fn precedence_of(token: &str) -> Option<u8> {
    match token {
        // "." | "[" | "(" | "^" => Some(6),
        "*" | "/" | "//" | "%" => Some(5),
        "+" | "-" | "++" => Some(4),
        "==" | "!=" | "<" | ">" | "<=" | ">=" => Some(3),
        "&&" | "&" => Some(2),
        "||" | "|" => Some(1),
        // "<-" => Some(0),
        _ => None,
    }
}
pub(crate) fn infix_expr_op<'a, I, T>(
    precedence: u8,
) -> impl Parser<I, Output = impl Fn(Expr<'a, T>, Expr<'a, T>) -> Expr<'a, T>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    // TODO: instead of testing every possible operator, just output a parser of
    // operator with given precedence, be wary of assignment operator `<-`, and
    // range operators
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
            Some(this_precedence) if this_precedence == precedence => Ok(token),
            // TODO: these errors are made for attempt combinator outside, it
            // should be silent
            Some(_) => Err(<StreamErrorFor<I>>::unexpected_static_message(
                "operator with equal precedence",
            )),
            None => Err(<StreamErrorFor<I>>::expected_static_message(
                "expression operator",
            )),
        })
        .map(|op| {
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
                _ => unreachable!(),
            };
            move |left, right| {
                Expr::Binary(Binary {
                    kind: op,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
        })
}
