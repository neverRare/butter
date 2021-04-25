use crate::expr::compound::Struct;
use crate::expr::operator::Assign;
use crate::expr::operator::Binary;
use crate::expr::operator::NamedArgCall;
use crate::expr::operator::Property;
use crate::expr::operator::Slice;
use crate::expr::operator::UnnamedArgCall;
use crate::expr::range::Range;
use crate::expr::PlaceExpr;
use crate::parser::expr::array::range;
use crate::parser::expr::expr;
use crate::parser::expr::record;
use crate::parser::expr::Expr;
use crate::parser::ident_keyword::ident;
use crate::parser::lex;
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

pub enum PartialAst<'a> {
    Property(&'a str),
    OptionalProperty(&'a str),
    Index(Expr<'a>),
    OptionalIndex(Expr<'a>),
    Slice(Range<'a>),
    OptionalSlice(Range<'a>),
    NamedArgCall(Struct<'a>),
    UnnamedArgCall(Box<[Expr<'a>]>),
}
impl<'a> PartialAst<'a> {
    pub fn combine_from(self, left: Expr<'a>) -> Expr<'a> {
        match self {
            Self::Property(name) => Expr::Property(Property {
                expr: Box::new(left),
                name,
            }),
            Self::OptionalProperty(name) => Expr::OptionalProperty(Property {
                expr: Box::new(left),
                name,
            }),
            Self::Index(index) => Expr::Index(Binary {
                left: Box::new(left),
                right: Box::new(index),
            }),
            Self::OptionalIndex(index) => Expr::OptionalIndex(Binary {
                left: Box::new(left),
                right: Box::new(index),
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
        }
    }
}
fn infix_7<'a, I>() -> impl Parser<I, Output = PartialAst<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    // TODO: disallow variable but allow (variable) possibly with lookahead
    let nameless_arg = || expr(0);
    let nameless_args = || {
        between(
            lex(char('(')),
            lex(char(')')),
            sep_end_by(nameless_arg(), lex(char(','))),
        )
        .map(<Vec<_>>::into)
    };
    let property = || {
        lex(attempt(
            char('.')
                .skip(not_followed_by(char('<')))
                .skip(not_followed_by(char('.'))),
        ))
        .with(lex(ident()))
        .map(PartialAst::Property)
    };
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
    choice((
        attempt(record()).map(PartialAst::NamedArgCall),
        nameless_args().map(PartialAst::UnnamedArgCall),
        property_index_slice(),
        optional(),
    ))
}
pub fn expr_7<'a, I>() -> impl Parser<I, Output = Expr<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (expr(8), many(infix_7())).map(|(prefix, infixes)| {
        let infixes: Vec<_> = infixes;
        let mut expr = prefix;
        for infix in infixes {
            expr = infix.combine_from(expr);
        }
        expr
    })
}
pub fn expr_0<'a, I>() -> impl Parser<I, Output = Expr<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
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
pub fn infix_op<'a, I>(
    precedence: u8,
) -> impl Parser<I, Output = fn(Expr<'a>, Expr<'a>) -> Expr<'a>>
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
            attempt(string("??")),
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
            char('?'),
            char('('),
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
    let precedence_checker = move |token| match precedence_of(token) {
        Some(this_precedence) if this_precedence > precedence => Ok(token),
        Some(_) => Err(<StreamErrorFor<I>>::unexpected_static_message(
            "infix operator with higher precedence",
        )),
        None => Err(<StreamErrorFor<I>>::expected_static_message(
            "expression operator",
        )),
    };
    macro_rules! op_matcher {
        ($(($str:pat, $name:ident $(,)?)),* $(,)?) => {
            |op| match op {
                $($str => {
                    let fun: fn(_, _) -> _ = |left, right| {
                        Expr::$name(Binary {
                            left: Box::new(left),
                            right: Box::new(right),
                        })
                    };
                    fun
                })*
                _ => unreachable!(),
            }
        };
    }
    lex(attempt(
        choice((double_ops(), recognize(single_ops()))).and_then(precedence_checker),
    ))
    .map(op_matcher![
        ("+", Add),
        ("-", Sub),
        ("*", Multiply),
        ("/", Div),
        ("//", FloorDiv),
        ("%", Mod),
        ("&", And),
        ("|", Or),
        ("||", LazyOr),
        ("??", NullOr),
        ("==", Equal),
        ("!=", NotEqual),
        ("<", Less),
        (">", Greater),
        ("<=", LessEqual),
        (">=", GreaterEqual),
        ("++", Concatenate),
        ("&&", LazyAnd),
    ])
}
