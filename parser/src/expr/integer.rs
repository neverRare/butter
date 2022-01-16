use combine::{
    attempt, choice,
    error::StreamError,
    many1, not_followed_by,
    parser::{
        char::{alpha_num, char},
        combinator::recognize,
    },
    satisfy, skip_many,
    stream::StreamErrorFor,
    ParseError, Parser, Stream,
};
use string_cache::DefaultAtom;

pub(crate) fn parse_digit(ch: char, base: u8) -> Option<u8> {
    let (lower_ch, lower_bound) = match ch {
        '0'..='9' => ('0', 0),
        'a'..='z' => ('a', 10),
        'A'..='Z' => ('A', 10),
        _ => return None,
    };
    let result = ch as u8 - lower_ch as u8 + lower_bound;
    if result < base {
        Some(result)
    } else {
        None
    }
}
macro_rules! gen_integer_decoder {
    ($ident:ident, $type:ty) => {
        pub(crate) fn $ident(src: &str, base: $type) -> Option<$type> {
            let mut result: $type = 0;
            for ch in src.chars().filter(|ch| *ch != '_') {
                let digit = parse_digit(ch, base as u8).unwrap() as $type;
                result = result
                    .checked_mul(base)
                    .and_then(|result| result.checked_add(digit))?;
            }
            Some(result)
        }
    };
}
gen_integer_decoder!(parse_u64, u64);
gen_integer_decoder!(parse_i64, i64);
pub(crate) fn integer_str<I>(base: u8) -> impl Parser<I, Output = DefaultAtom>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    recognize((
        satisfy(move |ch| parse_digit(ch, base).is_some()),
        skip_many(satisfy(move |ch| {
            parse_digit(ch, base).is_some() || ch == '_'
        })),
    ))
    .map(|int: String| DefaultAtom::from(int))
}
pub(crate) fn integer_str_allow_underscore<I>(base: u8) -> impl Parser<I, Output = DefaultAtom>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    many1(satisfy(move |ch| {
        parse_digit(ch, base).is_some() || ch == '_'
    }))
    .map(|int: String| DefaultAtom::from(int))
}
// TODO: clean this up, it looks ugly
macro_rules! gen_integer_parser {
    ($ident:ident, $parser:expr, $type:ty) => {
        combine::parser! {
            pub(crate) fn $ident[I]()(I) -> $type
            where [
                I: Stream<Token = char>,
                I::Error: ParseError<I::Token, I::Range, I::Position>,
            ] {
                combine::parser! {
                    fn integer[I, P](num_parser: P, base: $type)(I) -> $type
                    where [
                        I: Stream<Token = char>,
                        I::Error: ParseError<I::Token, I::Range, I::Position>,
                        P: Parser<I, Output = DefaultAtom>
                    ] {
                        num_parser
                            .and_then(|src: DefaultAtom| {
                                match $parser(src.as_ref(), *base) {
                                    Some(result) => Ok(result),
                                    None => Err(<StreamErrorFor<I>>::message_static_message("integer overflow")),
                                }
                            })
                    }
                }
                let base_prefix = |lower, upper| (char('0'), choice([char(lower), char(upper)]));
                choice((
                    attempt(base_prefix('x', 'X'))
                        .with(integer(integer_str_allow_underscore(16), 16)),
                    attempt(base_prefix('o', 'O'))
                        .with(integer(integer_str_allow_underscore(8), 8)),
                    attempt(base_prefix('b', 'B'))
                        .with(integer(integer_str_allow_underscore(2), 2)),
                    integer(integer_str(10), 10),
                ))
                    .skip(not_followed_by(alpha_num()))
                    .expected("integer")
            }
        }
    };
}
gen_integer_parser!(integer_u64, parse_u64, u64);
gen_integer_parser!(integer_i64, parse_i64, i64);
#[cfg(test)]
mod test {
    use crate::expr::integer_u64;
    use combine::EasyParser;

    #[test]
    fn decimal() {
        let src = "123";
        let expected = 123;
        assert_eq!(integer_u64().easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn hex() {
        let src = "0x_12e";
        let expected = 0x12e;
        assert_eq!(integer_u64().easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn oct() {
        let src = "0o_127";
        let expected = 0o127;
        assert_eq!(integer_u64().easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn bin() {
        let src = "0b_11110000";
        let expected = 0b11110000;
        assert_eq!(integer_u64().easy_parse(src), Ok((expected, "")));
    }
}
