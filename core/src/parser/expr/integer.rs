use combine::attempt;
use combine::choice;
use combine::error::StreamError;
use combine::not_followed_by;
use combine::parser;
use combine::parser::char::alpha_num;
use combine::parser::char::char;
use combine::parser::range::recognize;
use combine::parser::range::take_while;
use combine::parser::range::take_while1;
use combine::satisfy;
use combine::stream::StreamErrorFor;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;

pub fn parse_digit(ch: char, base: u8) -> Option<u8> {
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
macro_rules! gen_integer_parser {
    ($ident:ident, $type:ty) => {
        pub fn $ident(src: &str, base: $type) -> Option<$type> {
            let mut result: $type = 0;
            for ch in src.chars().filter(|ch| *ch != '_') {
                let digit = parse_digit(ch, base as u8).unwrap() as $type;
                let new_result = result
                    .checked_mul(base)
                    .and_then(|result| result.checked_add(digit));
                if let Some(new_result) = new_result {
                    result = new_result;
                } else {
                    return None;
                }
            }
            Some(result)
        }
    };
}
gen_integer_parser!(parse_u64, u64);
gen_integer_parser!(parse_i32, i32);
pub fn integer_str<'a, I>(base: u64) -> impl Parser<I, Output = &'a str>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    recognize((
        satisfy(move |ch| parse_digit(ch, base as u8).is_some()),
        take_while(move |ch| parse_digit(ch, base as u8).is_some() || ch == '_'),
    ))
}
pub fn integer_str_allow_underscore<'a, I>(base: u64) -> impl Parser<I, Output = &'a str>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    take_while1(move |ch| parse_digit(ch, base as u8).is_some() || ch == '_')
}
parser! {
    fn integer['a, I, P](num_parser: P, base: u64)(I) -> u64
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
        P: Parser<I, Output = &'a str>
    ] {
        num_parser
            .and_then(|src: &str| {
                match parse_u64(src, *base) {
                    Some(result) => Ok(result),
                    None => Err(<StreamErrorFor<I>>::message_static_message("integer overflow"))
                }
            })
    }
}
parser! {
    pub fn based_integer['a, I]()(I) -> u64
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
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
    }
}
#[cfg(test)]
mod test {
    use crate::parser::expr::based_integer;
    use combine::EasyParser;

    #[test]
    fn decimal() {
        let src = "123";
        let expected = 123;
        assert_eq!(based_integer().easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn hex() {
        let src = "0x_12e";
        let expected = 0x12e;
        assert_eq!(based_integer().easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn oct() {
        let src = "0o_127";
        let expected = 0o127;
        assert_eq!(based_integer().easy_parse(src), Ok((expected, "")));
    }
    #[test]
    fn bin() {
        let src = "0b_11110000";
        let expected = 0b11110000;
        assert_eq!(based_integer().easy_parse(src), Ok((expected, "")));
    }
}
