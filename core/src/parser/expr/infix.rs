use crate::ast::expr::compound::Arg;
use crate::ast::expr::operator::Assign;
use crate::ast::expr::operator::Binary;
use crate::ast::expr::operator::Call;
use crate::ast::expr::operator::Property;
use crate::ast::expr::operator::Slice;
use crate::ast::expr::range::Range;
use crate::ast::expr::PlaceExpr;
use crate::parser::expr::expr;
use crate::parser::expr::Expr;
use crate::parser::ident_keyword::ident;
use crate::parser::lex;
use combine::attempt;
use combine::choice;
use combine::look_ahead;
use combine::parser;
use combine::parser::char::char;
use combine::parser::char::string;
use combine::satisfy;
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
    Call(Arg<'a>),
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
            Self::Call(args) => Expr::Call(Call {
                expr: Box::new(left),
                args,
            }),
            _ => unreachable!(),
        })
    }
}
// `.` `?.` element access or slice `[...]` `?[...]` function call `(...)`
pub fn infix_7<'a, I>() -> impl Parser<I, Output = PartialAst<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    // HACK: avoid range operators
    let dot_not_range = || (char('.'), look_ahead(satisfy(|ch| ch != '.' || ch != '<')));
    choice(((attempt(lex(dot_not_range())), ident()).map(|(_, name)| PartialAst::Property(name)),))
}
// `*` `/` `//` `%`
pub fn infix_6<'a, I>() -> impl Parser<I, Output = PartialAst<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        (attempt(lex(string("//"))), expr(infix_7())).map(|(_, expr)| PartialAst::FloorDiv(expr)),
        infix_7(),
        (lex(char('*')), expr(infix_7())).map(|(_, expr)| PartialAst::Multiply(expr)),
        (lex(char('/')), expr(infix_7())).map(|(_, expr)| PartialAst::Div(expr)),
        (lex(char('%')), expr(infix_7())).map(|(_, expr)| PartialAst::Mod(expr)),
    ))
}
// `+` `-` `++`
pub fn infix_5<'a, I>() -> impl Parser<I, Output = PartialAst<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        (attempt(lex(string("++"))), expr(infix_6()))
            .map(|(_, expr)| PartialAst::Concatenate(expr)),
        infix_6(),
        (lex(char('+')), expr(infix_6())).map(|(_, expr)| PartialAst::Add(expr)),
        (lex(char('-')), expr(infix_6())).map(|(_, expr)| PartialAst::Sub(expr)),
    ))
}
// `==` `!=` `<` `>` `<=` `>=`
pub fn infix_4<'a, I>() -> impl Parser<I, Output = PartialAst<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    // HACK: avoid range operators
    let greater_not_range = || (char('>'), look_ahead(satisfy(|ch| ch != '.' || ch != '<')));
    choice((
        (attempt(lex(string("=="))), expr(infix_5())).map(|(_, expr)| PartialAst::Equal(expr)),
        (attempt(lex(string("!="))), expr(infix_5())).map(|(_, expr)| PartialAst::NotEqual(expr)),
        (attempt(lex(string("<="))), expr(infix_5())).map(|(_, expr)| PartialAst::LessEqual(expr)),
        (attempt(lex(string(">="))), expr(infix_5()))
            .map(|(_, expr)| PartialAst::GreaterEqual(expr)),
        (attempt(lex(greater_not_range())), expr(infix_5()))
            .map(|(_, expr)| PartialAst::Greater(expr)),
        infix_5(),
        (lex(char('<')), expr(infix_5())).map(|(_, expr)| PartialAst::Less(expr)),
    ))
}
// `&` `&&`
pub fn infix_3<'a, I>() -> impl Parser<I, Output = PartialAst<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        (attempt(lex(string("&&"))), expr(infix_4())).map(|(_, expr)| PartialAst::LazyAnd(expr)),
        infix_4(),
        (lex(char('&')), expr(infix_4())).map(|(_, expr)| PartialAst::And(expr)),
    ))
}
// `|` `||`
pub fn infix_2<'a, I>() -> impl Parser<I, Output = PartialAst<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        (attempt(lex(string("||"))), expr(infix_3())).map(|(_, expr)| PartialAst::LazyOr(expr)),
        infix_3(),
        (lex(char('|')), expr(infix_3())).map(|(_, expr)| PartialAst::Or(expr)),
    ))
}
// `??`
pub fn infix_1<'a, I>() -> impl Parser<I, Output = PartialAst<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        (attempt(lex(string("??"))), expr(infix_2())).map(|(_, expr)| PartialAst::NullOr(expr)),
        infix_2(),
    ))
}
// `<-`
parser! {
    pub fn infix_0['a, I]()(I) -> PartialAst<'a>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        choice((
            (attempt(lex(string("<-"))), expr(infix_0())).map(|(_, expr)| PartialAst::Assign(expr)),
            infix_1(),
        ))
    }
}
