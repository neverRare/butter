use crate::ast::expr::compound::Struct;
use crate::ast::expr::operator::Assign;
use crate::ast::expr::operator::Binary;
use crate::ast::expr::operator::NamedArgCall;
use crate::ast::expr::operator::Property;
use crate::ast::expr::operator::Slice;
use crate::ast::expr::operator::UnnamedArgCall;
use crate::ast::expr::range::Range;
use crate::ast::expr::PlaceExpr;
use crate::parser::expr::array::range;
use crate::parser::expr::expr;
use crate::parser::expr::record;
use crate::parser::expr::Expr;
use crate::parser::ident_keyword::ident;
use crate::parser::lex;
use crate::parser::sep_optional_end_by;
use combine::any;
use combine::attempt;
use combine::between;
use combine::choice;
use combine::error::StreamError;
use combine::look_ahead;
use combine::parser::char::char;
use combine::parser::char::string;
use combine::parser::range::recognize;
use combine::stream::StreamErrorFor;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

pub enum PartialAst<'a> {
    Add(Expr<'a>),
    Sub(Expr<'a>),
    Multiply(Expr<'a>),
    Div(Expr<'a>),
    FloorDiv(Expr<'a>),
    Mod(Expr<'a>),
    And(Expr<'a>),
    Or(Expr<'a>),
    LazyAnd(Expr<'a>),
    LazyOr(Expr<'a>),
    Equal(Expr<'a>),
    NotEqual(Expr<'a>),
    Greater(Expr<'a>),
    GreaterEqual(Expr<'a>),
    Less(Expr<'a>),
    LessEqual(Expr<'a>),
    Concatenate(Expr<'a>),
    NullOr(Expr<'a>),

    Assign(Expr<'a>),

    Property(&'a str),
    OptionalProperty(&'a str),
    Index(Expr<'a>),
    OptionalIndex(Expr<'a>),
    Slice(Range<'a>),
    OptionalSlice(Range<'a>),
    NamedArgCall(Struct<'a>),
    UnnamedArgCall(Vec<Expr<'a>>),
}
impl<'a> PartialAst<'a> {
    // None means <- is applied to non-place expression
    pub fn combine_from(self, left: Expr<'a>) -> Option<Expr<'a>> {
        macro_rules! binary {
            ($left:ident, $infix:ident, [$($ident:ident),* $(,)?] $(,)?) => {{
                $(
                    if let Self::$ident(right) = $infix {
                        return Some(Expr::$ident(Binary {
                            left: Box::new($left),
                            right: Box::new(right),
                        }))
                    }
                )*
            };
        }}
        binary!(
            left,
            self,
            [
                Add,
                Sub,
                Multiply,
                Div,
                FloorDiv,
                Mod,
                And,
                Or,
                LazyAnd,
                LazyOr,
                Equal,
                NotEqual,
                Greater,
                GreaterEqual,
                Less,
                LessEqual,
                Concatenate,
                NullOr,
                Index,
                OptionalIndex,
            ],
        );
        Some(match self {
            Self::Assign(right) => Expr::Assign(Assign {
                place: Box::new(PlaceExpr::from_expr(left)?),
                expr: Box::new(right),
            }),
            Self::Property(name) => Expr::Property(Property {
                expr: Box::new(left),
                name,
            }),
            Self::OptionalProperty(name) => Expr::OptionalProperty(Property {
                expr: Box::new(left),
                name,
            }),
            Self::Slice(range) => Expr::Slice(Slice {
                expr: Box::new(left),
                range,
            }),
            Self::OptionalSlice(range) => Expr::OptionalSlice(Slice {
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
            _ => unreachable!(),
        })
    }
}
pub fn full_infix<'a, I>() -> impl Parser<I, Output = PartialAst<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let property = || lex(char('.')).with(lex(ident())).map(PartialAst::Property);
    let index = || between(lex(char('[')), lex(char(']')), expr(0)).map(PartialAst::Index);
    let property_index_slice =
        || choice((property(), attempt(index()), range().map(PartialAst::Slice)));
    let optional = || {
        lex(char('?'))
            .with(property_index_slice())
            .map(|infix| match infix {
                PartialAst::Property(name) => PartialAst::OptionalProperty(name),
                PartialAst::Index(index) => PartialAst::OptionalIndex(index),
                PartialAst::Slice(range) => PartialAst::OptionalSlice(range),
                _ => unreachable!(),
            })
    };
    let nameless_arg = || {
        expr(0).and_then(|expr| {
            if let Expr::Var(_) = expr {
                Err(<StreamErrorFor<I>>::unexpected_static_message(
                    "mixed named and unnamed argument",
                ))
            } else {
                Ok(expr)
            }
        })
    };
    let nameless_args = || {
        between(
            lex(char('(')),
            lex(char(')')),
            sep_optional_end_by(nameless_arg, || lex(char(','))),
        )
    };
    choice((
        attempt(lex(string("//")))
            .with(expr(7))
            .map(PartialAst::FloorDiv),
        attempt(lex(string("++")))
            .with(expr(6))
            .map(PartialAst::Concatenate),
        attempt(lex(string("==")))
            .with(expr(5))
            .map(PartialAst::Equal),
        attempt(lex(string("!=")))
            .with(expr(5))
            .map(PartialAst::NotEqual),
        attempt(lex(string("<=")))
            .with(expr(5))
            .map(PartialAst::LessEqual),
        attempt(lex(string(">=")))
            .with(expr(5))
            .map(PartialAst::GreaterEqual),
        attempt(lex(string("&&")))
            .with(expr(4))
            .map(PartialAst::LazyAnd),
        attempt(lex(string("||")))
            .with(expr(3))
            .map(PartialAst::LazyOr),
        attempt(lex(string("??")))
            .with(expr(2))
            .map(PartialAst::NullOr),
        attempt(lex(string("<-")))
            .with(expr(0))
            .map(PartialAst::Assign),
        lex(char('|')).with(expr(3)).map(PartialAst::Or),
        lex(char('&')).with(expr(4)).map(PartialAst::And),
        lex(char('<')).with(expr(5)).map(PartialAst::Less),
        lex(char('>')).with(expr(5)).map(PartialAst::Greater),
        lex(char('+')).with(expr(6)).map(PartialAst::Add),
        lex(char('-')).with(expr(6)).map(PartialAst::Sub),
        lex(char('*')).with(expr(7)).map(PartialAst::Multiply),
        lex(char('/')).with(expr(7)).map(PartialAst::Div),
        lex(char('%')).with(expr(7)).map(PartialAst::Mod),
        property_index_slice(),
        optional(),
        attempt(record()).map(PartialAst::NamedArgCall),
        nameless_args().map(PartialAst::UnnamedArgCall),
    ))
}
pub fn infix<'a, I>(precedence: u8) -> impl Parser<I, Output = PartialAst<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let checker = move |token| match precedence_of(token) {
        Some(this_precedence) if this_precedence > precedence => Ok(token),
        _ => Err(<StreamErrorFor<I>>::expected_static_message(
            "infix operator",
        )),
    };
    let single_operator = || recognize(any()).and_then(checker);
    let double_operator = || recognize((any(), any())).and_then(checker);
    let valid_operator = || attempt(double_operator()).or(single_operator());
    look_ahead(valid_operator())
        // TODO: resolve the following to ignore range operators, this leads to cryptic error >:(
        // .with(not_followed_by(range_operator()))
        .with(full_infix())
}
pub fn precedence_of(token: &str) -> Option<u8> {
    match token {
        "." | "[" | "?" | "(" => Some(8),
        "*" | "/" | "//" | "%" => Some(7),
        "+" | "-" | "++" => Some(6),
        "==" | "!=" | "<" | ">" | "<=" | ">=" => Some(5),
        "&&" | "&" => Some(4),
        "||" | "|" => Some(3),
        "??" => Some(2),
        "<-" => Some(1),
        _ => None,
    }
}
