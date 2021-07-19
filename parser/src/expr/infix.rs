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
use hir::expr::operator::Binary;
use hir::expr::operator::NamedArgCall;
use hir::expr::operator::Property;
use hir::expr::operator::Slice;
use hir::expr::operator::UnnamedArgCall;
use hir::expr::range::Range;
use hir::expr::PlaceExpr;

pub enum PartialAst<'a> {
    Property(&'a str),
    Index(Expr<'a, ()>),
    Slice(Range<'a, ()>),
    NamedArgCall(Record<'a, ()>),
    UnnamedArgCall(Box<[Expr<'a, ()>]>),
    Deref,
    Len,
}
impl<'a> PartialAst<'a> {
    pub fn combine_from(self, left: Expr<'a, ()>) -> Expr<'a, ()> {
        match self {
            Self::Property(name) => Expr::Property(Property {
                expr: Box::new(left),
                name,
            }),
            Self::Index(index) => Expr::Index(Binary {
                left: Box::new(left),
                right: Box::new(index),
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
fn infix_6<'a, I>() -> impl Parser<I, Output = PartialAst<'a>>
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
pub fn expr_6<'a, I>() -> impl Parser<I, Output = Expr<'a, ()>>
where
    I: RangeStream<Token = char, Range = &'a str>,
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
pub fn expr_0<'a, I>() -> impl Parser<I, Output = Expr<'a, ()>>
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
pub fn infix_expr_op<'a, I>(
    precedence: u8,
) -> impl Parser<I, Output = fn(Expr<'a, ()>, Expr<'a, ()>) -> Expr<'a, ()>>
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
        ($($str:pat => $name:ident),* $(,)?) => {
            |op| match op {
                $($str => {
                    let fun: fn(_, _) -> _ = |left, right| {
                        Expr::$name(Binary {
                            left: Box::new(left),
                            right: Box::new(right),
                        })
                    };
                    Ok(fun)
                })*
                _ => Err(<StreamErrorFor<I>>::expected_static_message(
                    "infix expression operator",
                )),
            }
        };
    }
    lex(choice((double_ops(), recognize(single_ops()))))
        .and_then(precedence_checker)
        .and_then(op_matcher! {
            "+" => Add,
            "-" => Sub,
            "*" => Multiply,
            "/" => Div,
            "//" => FloorDiv,
            "%" => Mod,
            "&" => And,
            "|" => Or,
            "||" => LazyOr,
            "==" => Equal,
            "!=" => NotEqual,
            "<" => Less,
            ">" => Greater,
            "<=" => LessEqual,
            ">=" => GreaterEqual,
            "++" => Concatenate,
            "&&" => LazyAnd,
        })
}
