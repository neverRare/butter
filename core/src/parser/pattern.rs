use crate::ast::pattern::ArrayWithRest;
use crate::ast::pattern::Pattern;
use crate::ast::pattern::StructPattern;
use crate::parser::ident_keyword::ident;
use crate::parser::ident_keyword::keyword;
use crate::parser::lex;
use combine::attempt;
use combine::choice;
use combine::optional;
use combine::parser;
use combine::parser::char::char;
use combine::sep_by;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;
use std::collections::HashMap;

// TODO: handle trailing comma

fn elements<'a, I>() -> impl Parser<I, Output = Vec<Pattern<'a>>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    sep_by(pattern(), lex(char(',')))
}
fn rest<'a, I>() -> impl Parser<I, Output = Pattern<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (lex(char('*')), pattern()).map(|(_, pattern)| pattern)
}
fn array<'a, I>() -> impl Parser<I, Output = Pattern<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        lex(char('[')),
        elements(),
        optional((rest(), optional((lex(char(',')), elements())))),
        lex(char(']')),
    )
        .map(|(_, left, rest_right, _)| match rest_right {
            Some((rest, right)) => {
                let right = match right {
                    Some((_, right)) => right,
                    None => vec![],
                };
                Pattern::ArrayWithRest(ArrayWithRest {
                    left,
                    rest: Box::new(rest),
                    right,
                })
            }
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
fn fields<'a, I>() -> impl Parser<I, Output = HashMap<&'a str, Pattern<'a>>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    // TODO: handle duplicated name as error
    sep_by(field(), lex(char(',')))
}
fn record<'a, I>() -> impl Parser<I, Output = StructPattern<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        lex(char('(')),
        fields(),
        optional((rest(), optional((lex(char(',')), fields())))),
        lex(char(')')),
    )
        .map(|(_, left, rest_right, _)| match rest_right {
            Some((rest, right)) => {
                let fields = match right {
                    Some((_, right)) => {
                        let mut fields = left;
                        fields.extend(right);
                        fields
                    }
                    None => left,
                };
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
        array(),
        record().map(Pattern::Struct),
        attempt(lex(keyword("_"))).map(|_| Pattern::Ignore),
        lex(ident()).map(Pattern::Var),
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
    array = [first, second],
    array_with_rest = [first, *rest, last]
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
