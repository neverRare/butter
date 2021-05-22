use crate::expr::control_flow::Param;
use crate::parser::ident_keyword::ident;
use crate::parser::ident_keyword::keyword;
use crate::parser::lex;
use crate::pattern::ArrayWithRest;
use crate::pattern::Pattern;
use crate::pattern::StructPattern;
use crate::pattern::TaggedPattern;
use crate::pattern::Var;
use combine::attempt;
use combine::between;
use combine::choice;
use combine::many;
use combine::optional;
use combine::parser;
use combine::parser::char::char;
use combine::sep_end_by;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;
use std::collections::HashMap;

fn optional_rest<'a, I, EP, RP, C>(
    left: char,
    right: char,
    element: fn() -> EP,
    rest: fn() -> RP,
) -> impl Parser<I, Output = (C, Option<(RP::Output, C)>)>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
    EP: Parser<I>,
    RP: Parser<I>,
    C: Extend<EP::Output> + Default,
{
    let no_rest = move || sep_end_by(element(), lex(char(',')));
    let have_rest = move || {
        (
            many(element().skip(lex(char(',')))),
            (lex(char('*'))).with(rest()),
            optional(lex(char(',')).with(no_rest()))
                .map(|right| right.unwrap_or_else(Default::default)),
        )
    };
    let middle = move || {
        choice((
            attempt(have_rest()).map(|(left, rest, right)| (left, Some((rest, right)))),
            no_rest().map(|collection| (collection, None)),
        ))
    };
    between(lex(char(left)), lex(char(right)), middle())
}
fn array<'a, I>() -> impl Parser<I, Output = Pattern<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    optional_rest('[', ']', pattern, pattern).map(|(left, rest_right)| {
        let left: Vec<_> = left;
        match rest_right {
            Some((rest, right)) => Pattern::ArrayWithRest(ArrayWithRest {
                left: left.into(),
                rest: Box::new(rest),
                right: right.into(),
            }),
            None => Pattern::Array(left.into()),
        }
    })
}
fn field<'a, I>() -> impl Parser<I, Output = (&'a str, Pattern<'a>)>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (lex(ident()), optional(lex(char('=')).with(pattern()))).map(|(name, pattern)| {
        (
            name,
            pattern.unwrap_or(Pattern::Var(Var {
                ident: name,
                mutable: false,
                bind_to_ref: false,
            })),
        )
    })
}
#[derive(Debug, PartialEq, Clone, Default)]
pub struct ParamExtend<'a> {
    order: Vec<&'a str>,
    param: HashMap<&'a str, Pattern<'a>>,
}
impl<'a> ParamExtend<'a> {
    fn into_param(self) -> Param<'a> {
        let mut param = self.param;
        param.shrink_to_fit();
        Param {
            order: self.order.into(),
            param,
        }
    }
}
impl<'a> Extend<(&'a str, Pattern<'a>)> for ParamExtend<'a> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (&'a str, Pattern<'a>)>,
    {
        let iter = iter.into_iter();
        let (min, _) = iter.size_hint();
        self.order.reserve(min);
        self.param.reserve(min);
        for (name, pattern) in iter {
            self.order.push(name);
            self.param.insert(name, pattern);
        }
    }
}
pub fn parameter<'a, I>() -> impl Parser<I, Output = Param<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    between(
        lex(char('(')),
        lex(char(')')),
        sep_end_by(field(), lex(char(','))).map(ParamExtend::into_param),
    )
}
fn record<'a, I>() -> impl Parser<I, Output = StructPattern<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    // TODO: handle duplicate name as error
    optional_rest('(', ')', field, pattern).map(|(left, rest_right)| match rest_right {
        Some((rest, right)) => {
            let mut fields: HashMap<_, _> = left;
            fields.extend(right);
            StructPattern {
                fields,
                rest: Some(Box::new(rest)),
            }
        }
        None => StructPattern {
            fields: left,
            rest: None,
        },
    })
}
fn pattern_<'a, I>() -> impl Parser<I, Output = Pattern<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        lex(char('@'))
            .with((lex(ident()), optional(pattern())))
            .map(|(tag, pattern)| {
                Pattern::Tag(TaggedPattern {
                    tag,
                    pattern: pattern.map(Box::new),
                })
            }),
        lex(char('&'))
            .with((optional(lex(keyword("mut"))), pattern()))
            .map(|(mutability, pattern)| match mutability {
                Some(_) => Pattern::RefMut(Box::new(pattern)),
                None => Pattern::Ref(Box::new(pattern)),
            }),
        array(),
        attempt(between(lex(char('(')), lex(char(')')), pattern())),
        record().map(Pattern::Struct),
        attempt(lex(keyword("_"))).map(|_| Pattern::Ignore),
        (
            optional(attempt(lex(keyword("ref")))),
            optional(attempt(lex(keyword("mut")))),
            lex(ident()),
        )
            .map(|(bind_to_ref, mutability, ident)| {
                Pattern::Var(Var {
                    ident,
                    mutable: mutability.is_some(),
                    bind_to_ref: bind_to_ref.is_some(),
                })
            }),
    ))
}
parser! {
    pub fn pattern['a, I]()(I) -> Pattern<'a>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        pattern_()
    }
}
// #[cfg(test)]
// mod test {
//     use crate::parser::pattern::pattern;
//     use crate::parser::pattern::ArrayWithRest;
//     use crate::parser::pattern::Pattern;
//     use crate::parser::pattern::StructPattern;
//     use combine::EasyParser;
//     use std::array::IntoIter;

//     #[test]
//     fn parse_array() {
//         let src = "[first, second]";
//         let expected = Pattern::Array(vec![Pattern::Var("first"), Pattern::Var("second")].into());
//         assert_eq!(pattern().easy_parse(src), Ok((expected, "")));
//     }
//     #[test]
//     fn parse_array_with_rest() {
//         let src = "[first, *rest, last]";
//         let expected = Pattern::ArrayWithRest(ArrayWithRest {
//             left: vec![Pattern::Var("first")].into(),
//             rest: Box::new(Pattern::Var("rest")),
//             right: vec![Pattern::Var("last")].into(),
//         });
//         assert_eq!(pattern().easy_parse(src), Ok((expected, "")));
//     }
//     #[test]
//     fn parse_pattern() {
//         let src = "\
// (
//     shortcut,
//     var = foo,
//     ignore = _,
//     struct = (foo, *rest),
//     group = (foo),
//     array = [first, second],
//     array_with_rest = [first, *rest, last],
// )";
//         let expected = Pattern::Struct(StructPattern {
//             fields: IntoIter::new([
//                 ("shortcut", Pattern::Var("shortcut")),
//                 ("var", Pattern::Var("foo")),
//                 ("ignore", Pattern::Ignore),
//                 (
//                     "struct",
//                     Pattern::Struct(StructPattern {
//                         fields: IntoIter::new([("foo", Pattern::Var("foo"))]).collect(),
//                         rest: Some(Box::new(Pattern::Var("rest"))),
//                     }),
//                 ),
//                 ("group", Pattern::Var("foo")),
//                 (
//                     "array",
//                     Pattern::Array(vec![Pattern::Var("first"), Pattern::Var("second")].into()),
//                 ),
//                 (
//                     "array_with_rest",
//                     Pattern::ArrayWithRest(ArrayWithRest {
//                         left: vec![Pattern::Var("first")].into(),
//                         rest: Box::new(Pattern::Var("rest")),
//                         right: vec![Pattern::Var("last")].into(),
//                     }),
//                 ),
//             ])
//             .collect(),
//             rest: None,
//         });
//         assert_eq!(pattern().easy_parse(src), Ok((expected, "")));
//     }
// }
