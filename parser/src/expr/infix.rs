use crate::{
    expr::{array::range, expr, record::record, tuple::tuple},
    ident_keyword::ident,
    lex,
};
use combine::{
    attempt, between, choice,
    error::StreamError,
    many, not_followed_by, optional,
    parser::char::{char, string},
    stream::StreamErrorFor,
    value, ParseError, Parser, Stream,
};
use hir::{
    expr::{
        Arg, Assign, Binary, BinaryType, Call, Collection, Expr, ExprKind, Field, FieldAccess,
        Index, PlaceExpr, Range, Slice,
    },
    keyword, Atom,
};

#[derive(Debug, PartialEq, Clone)]
pub(super) enum PartialAst {
    Property(Atom),
    Index(Expr<()>),
    Slice(Range<()>),
    UnitCall,
    SplatCall(Expr<()>),
    RecordCall(Collection<Field<()>, ()>),
    TupleCall(Collection<Expr<()>, ()>),
    Deref,
    Len,
}
impl PartialAst {
    pub(super) fn combine_from(self, left: Expr<()>) -> Expr<()> {
        let res = match self {
            Self::Property(name) => ExprKind::Place(PlaceExpr::FieldAccess(FieldAccess {
                expr: Box::new(left),
                name,
            })),
            Self::Index(index) => ExprKind::Place(PlaceExpr::Index(Index {
                expr: Box::new(left),
                index: Box::new(index),
            })),
            Self::Slice(range) => ExprKind::Place(PlaceExpr::Slice(Slice {
                expr: Box::new(left),
                range,
            })),
            Self::UnitCall => ExprKind::Call(Call {
                expr: Box::new(left),
                arg: Arg::Unit,
            }),
            Self::SplatCall(arg) => ExprKind::Call(Call {
                expr: Box::new(left),
                arg: Arg::Splat(Box::new(arg)),
            }),
            Self::RecordCall(arg) => ExprKind::Call(Call {
                expr: Box::new(left),
                arg: Arg::Record(arg),
            }),
            Self::TupleCall(arg) => ExprKind::Call(Call {
                expr: Box::new(left),
                arg: Arg::Tuple(arg),
            }),
            Self::Deref => ExprKind::Place(PlaceExpr::Deref(Box::new(left))),
            Self::Len => ExprKind::Place(PlaceExpr::Len(Box::new(left))),
        };
        res.into_untyped()
    }
}
fn infix_6<I>() -> impl Parser<I, Output = PartialAst>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let property_or_len = || {
        lex(attempt(
            char('.')
                .skip(not_followed_by(char('<')))
                .skip(not_followed_by(char('.'))),
        ))
        .with(lex(ident()))
        .map(|prop| {
            if prop == keyword!("len") {
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
            attempt((lex(char('(')), lex(char(')')))).with(value(PartialAst::UnitCall)),
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
        lex(char('^')).with(value(PartialAst::Deref)),
    ))
}
pub(super) fn expr_6<I>() -> impl Parser<I, Output = Expr<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
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
pub(super) fn expr_0<I>() -> impl Parser<I, Output = Expr<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (expr(1), optional(lex(attempt(string("<-"))).with(expr(0)))).and_then(|(place, expr)| {
        match expr {
            Some(expr) => {
                if let ExprKind::Place(place) = place.expr {
                    Ok(ExprKind::Assign(vec![Assign { place, expr }].into()).into_untyped())
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
pub(super) fn infix_expr_op<I>(
    precedence: u8,
) -> impl Parser<I, Output = impl Fn(Expr<()>, Expr<()>) -> Expr<()>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let op = match precedence {
        5 => choice((
            attempt(string("//")).with(value(BinaryType::FloorDiv)),
            char('*').with(value(BinaryType::Multiply)),
            char('/').with(value(BinaryType::Div)),
            char('%').with(value(BinaryType::Mod)),
        ))
        .left()
        .left(),
        4 => choice((
            attempt(string("++")).with(value(BinaryType::Concatenate)),
            char('+').with(value(BinaryType::Add)),
            char('-').with(value(BinaryType::Sub)),
        ))
        .right()
        .left(),
        3 => choice((
            attempt(string("==")).with(value(BinaryType::Equal)),
            attempt(string("!=")).with(value(BinaryType::NotEqual)),
            attempt(string("<=")).with(value(BinaryType::LessEqual)),
            attempt(string(">=")).with(value(BinaryType::GreaterEqual)),
            attempt(char('<').skip(not_followed_by(char('-')))).with(value(BinaryType::Less)),
            attempt(char('>').skip(not_followed_by(choice([char('.'), char('<')]))))
                .with(value(BinaryType::Greater)),
        ))
        .left()
        .right(),
        2 => choice((
            attempt(string("&&")).with(value(BinaryType::LazyAnd)),
            char('&').with(value(BinaryType::And)),
        ))
        .right()
        .right(),
        1 => choice((
            attempt(string("||")).with(value(BinaryType::LazyOr)),
            char('|').with(value(BinaryType::Or)),
        ))
        .right()
        .right(),
        precedence => panic!("invalid precedence {}", precedence),
    };
    op.map(|op| {
        move |left, right| {
            ExprKind::Binary(Binary {
                kind: op,
                left: Box::new(left),
                right: Box::new(right),
            })
            .into_untyped()
        }
    })
}
