use crate::ast::pattern::ArrayWithRest;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::StructPattern;
use crate::parser::ident_keyword::ident;
use crate::parser::ident_keyword::keyword;
use crate::parser::lex;
use crate::parser::sep_optional_end_by;
use combine::attempt;
use combine::between;
use combine::choice;
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
    let no_rest = move || sep_optional_end_by(element, || lex(char(',')));
    let have_rest = move || {
        (
            sep_end_by(element(), lex(char(','))),
            (lex(char('*'))),
            rest(),
            optional((lex(char(',')), no_rest())),
        )
            .map(|(left, _, rest, right)| {
                let right = match right {
                    Some((_, right)) => right,
                    None => Default::default(),
                };
                (left, rest, right)
            })
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
    optional_rest('[', ']', pattern, pattern).map(|(left, rest_right)| match rest_right {
        Some((rest, right)) => Pattern::ArrayWithRest(ArrayWithRest {
            left,
            rest: Box::new(rest),
            right,
        }),
        None => Pattern::Array(left),
    })
}
fn field<'a, I>() -> impl Parser<I, Output = (&'a str, Pattern<'a>)>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (lex(ident()), optional((lex(char('=')), pattern()))).map(|(name, pattern)| {
        let pattern = match pattern {
            Some((_, pattern)) => pattern,
            None => Pattern::Var(name),
        };
        (name, pattern)
    })
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
parser! {
    pub fn pattern['a, I]()(I) -> Pattern<'a>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        choice((
            array(),
            attempt(between(lex(char('(')), lex(char(')')), pattern())),
            record().map(Pattern::Struct),
            attempt(lex(keyword("_"))).map(|_| Pattern::Ignore),
            lex(ident()).map(Pattern::Var),
        ))
    }
}
#[cfg(test)]
mod test {
    use crate::parser::pattern::pattern;
    use crate::parser::pattern::ArrayWithRest;
    use crate::parser::pattern::Pattern;
    use crate::parser::pattern::StructPattern;
    use combine::EasyParser;
    use std::array::IntoIter;

    #[test]
    fn parse_array() {
        let src = "[first, second]";
        let expected = Pattern::Array(vec![Pattern::Var("first"), Pattern::Var("second")]);
        assert_eq!(pattern().easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn parse_array_with_rest() {
        let src = "[first, *rest, last]";
        let expected = Pattern::ArrayWithRest(ArrayWithRest {
            left: vec![Pattern::Var("first")],
            rest: Box::new(Pattern::Var("rest")),
            right: vec![Pattern::Var("last")],
        });
        assert_eq!(pattern().easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn parse_pattern() {
        let src = "\
(
    shortcut,
    var = foo,
    ignore = _,
    struct = (foo, *rest),
    group = (foo),
    array = [first, second],
    array_with_rest = [first, *rest, last],
)";
        let expected = Pattern::Struct(StructPattern {
            fields: IntoIter::new([
                ("shortcut", Pattern::Var("shortcut")),
                ("var", Pattern::Var("foo")),
                ("ignore", Pattern::Ignore),
                (
                    "struct",
                    Pattern::Struct(StructPattern {
                        fields: IntoIter::new([("foo", Pattern::Var("foo"))]).collect(),
                        rest: Some(Box::new(Pattern::Var("rest"))),
                    }),
                ),
                ("group", Pattern::Var("foo")),
                (
                    "array",
                    Pattern::Array(vec![Pattern::Var("first"), Pattern::Var("second")]),
                ),
                (
                    "array_with_rest",
                    Pattern::ArrayWithRest(ArrayWithRest {
                        left: vec![Pattern::Var("first")],
                        rest: Box::new(Pattern::Var("rest")),
                        right: vec![Pattern::Var("last")],
                    }),
                ),
            ])
            .collect(),
            rest: None,
        });
        assert_eq!(pattern().easy_parse(src), Ok((expected, "")));
    }
}
