use crate::{
    expr::{array::range, expr, record::record, tuple::tuple},
    ident_keyword::ident,
    lex, size_of,
};
use combine::{
    attempt, between, choice,
    error::StreamError,
    many, not_followed_by, optional,
    parser::char::{char, string},
    stream::StreamErrorFor,
    value, ParseError, Parser, Stream,
};
use hir::expr::{
    Arg, Assign, Binary, BinaryType, Call, Expr, FieldAccess, Index, PlaceExpr, Range, Record,
    Slice, Tuple,
};
use string_cache::DefaultAtom;

pub(crate) enum PartialAst<T> {
    Property(DefaultAtom),
    Index(Expr<T>),
    Slice(Range<T>),
    UnitCall,
    SplatCall(Expr<T>),
    RecordCall(Record<T>),
    TupleCall(Tuple<T>),
    Deref,
    Len,
}
impl<T> PartialAst<T> {
    pub(crate) fn combine_from(self, left: Expr<T>) -> Expr<T> {
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
fn infix_6_<I, T>() -> impl Parser<I, Output = PartialAst<T>>
where
    I: Stream<Token = char>,
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
            if prop == DefaultAtom::from("len") {
                PartialAst::Len
            } else {
                PartialAst::Property(DefaultAtom::from(prop))
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
            .map(PartialAst::SplatCall),
            attempt(tuple()).map(PartialAst::TupleCall),
            record().map(PartialAst::RecordCall),
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
combine::parser! {
    fn infix_6[I, T]()(I) -> PartialAst< T>
    where [
        I: Stream<Token = char>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        T: Default,
    ] {
        infix_6_()
    }
}
pub(crate) fn expr_6<I, T>() -> impl Parser<I, Output = Expr<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    (expr(7), many(infix_6_())).map(|(prefix, infixes)| {
        let infixes: Vec<_> = infixes;
        let mut expr = prefix;
        for infix in infixes {
            expr = infix.combine_from(expr);
        }
        expr
    })
}
pub(crate) fn expr_0<I, T>() -> impl Parser<I, Output = Expr<T>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    T: Default,
{
    (expr(1), optional(lex(attempt(string("<-"))).with(expr(0)))).and_then(|(place, expr)| {
        match expr {
            Some(expr) => {
                if let Expr::Place(place) = place {
                    Ok(Expr::Assign(
                        vec![Assign {
                            place: Box::new(place),
                            expr: Box::new(expr),
                        }]
                        .into(),
                    ))
                } else {
                    Err(<StreamErrorFor<I>>::expected_static_message(
                        "place expression",
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
pub(crate) fn infix_expr_op<I, T>(
    precedence: u8,
) -> impl Parser<I, Output = impl Fn(Expr<T>, Expr<T>) -> Expr<T>>
where
    I: Stream<Token = char>,
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
            char('.').with(value(".")),
            char('[').with(value("[")),
            char('(').with(value("(")),
            char('^').with(value("^")),
            char('*').with(value("*")),
            char('/').with(value("/")),
            char('%').with(value("%")),
            char('+').with(value("+")),
            char('-').with(value("-")),
            char('<').with(value("<")),
            char('>').with(value(">")),
            char('&').with(value("&")),
            char('|').with(value("|")),
        ])
    };
    lex(choice((double_ops(), single_ops())))
        .and_then(move |token| match precedence_of(token) {
            Some(this_precedence) if this_precedence == precedence => Ok(token),
            // TODO: these errors are made for attempt combinator outside, it
            // should be silent
            Some(_) => Err(<StreamErrorFor<I>>::message_static_message(
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
pub(crate) fn print_infix_sizes() {
    println!(
        "{}: {}",
        concat!(module_path!(), "::infix_6_"),
        size_of(&infix_6_::<&str, ()>()),
    );
    println!(
        "{}: {}",
        concat!(module_path!(), "::expr_6"),
        size_of(&expr_6::<&str, ()>()),
    );
    println!(
        "{}: {}",
        concat!(module_path!(), "::expr_0"),
        size_of(&expr_0::<&str, ()>()),
    );
    println!(
        "{}: {}",
        concat!(module_path!(), "::infix_expr_op"),
        size_of(&infix_expr_op::<&str, ()>(0)),
    );
}
