use combine::{
    attempt, choice,
    error::StreamError,
    look_ahead, many1, not_followed_by, optional,
    parser::char::digit,
    parser::{
        char::{alpha_num, char},
        combinator::recognize,
    },
    satisfy, skip_many,
    stream::StreamErrorFor,
    value, ParseError, Parser, Stream,
};
use hir::{keyword, Atom};

// TODO: minus integer parser

pub(super) fn parse_digit(ch: char, base: u8) -> Option<u8> {
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
        pub(super) fn $ident(src: &str, base: $type) -> Option<$type> {
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
pub(super) fn integer_str<I>(base: u8) -> impl Parser<I, Output = Atom>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    recognize::<String, _, _>((
        satisfy(move |ch| parse_digit(ch, base).is_some()),
        skip_many(satisfy(move |ch| {
            parse_digit(ch, base).is_some() || ch == '_'
        })),
    ))
    .map(Atom::from)
}
pub(super) fn integer_str_allow_underscore<I>(base: u8) -> impl Parser<I, Output = Atom>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    many1::<String, _, _>(satisfy(move |ch| {
        parse_digit(ch, base).is_some() || ch == '_'
    }))
    .map(Atom::from)
}
macro_rules! gen_integer_parser {
    ($ident:ident, $parser:expr, $type:ty) => {
        pub(super) fn $ident<I>() -> impl Parser<I, Output = $type>
        where
            I: Stream<Token = char>,
            I::Error: ParseError<I::Token, I::Range, I::Position>,
        {
            let parse_mapper = |base| {
                move |src: Atom| {
                    $parser(src.as_ref(), base as $type).ok_or(
                        <StreamErrorFor<I>>::message_static_message("integer overflow"),
                    )
                }
            };
            let prefixed = choice([('x', 16), ('o', 8), ('b', 2)].map(|(prefix, base)| {
                attempt((
                    char('0'),
                    choice([
                        char(prefix.to_ascii_lowercase()),
                        char(prefix.to_ascii_uppercase()),
                    ]),
                ))
                .with(integer_str_allow_underscore(base).and_then(parse_mapper(base)))
            }));
            choice((prefixed, integer_str(10).and_then(parse_mapper(10))))
                .skip(not_followed_by(alpha_num()))
                .expected("integer")
        }
    };
}
gen_integer_parser!(integer_u64, parse_u64, u64);
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Sign {
    Plus,
    Minus,
}
impl Sign {
    fn into_char(self) -> char {
        match self {
            Self::Plus => '+',
            Self::Minus => '-',
        }
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
struct FloatSrc {
    whole: Atom,
    decimal: Atom,
    exp_sign: Sign,
    exp: Atom,
}
impl FloatSrc {
    fn parse(self) -> Option<f64> {
        // TODO: avoid string allocation, precision must be kept
        let src: String = format!(
            "0{}.{}0e{}0{}",
            self.whole.as_ref(),
            self.decimal.as_ref(),
            self.exp_sign.into_char(),
            self.exp.as_ref(),
        )
        .chars()
        .filter(|ch| *ch != '_')
        .collect();
        let float: f64 = src.parse().unwrap();
        if float.is_finite() {
            Some(float)
        } else {
            None
        }
    }
}
fn float_src<I>() -> impl Parser<I, Output = FloatSrc>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let sign = || {
        choice((
            char('+').with(value(Sign::Plus)),
            char('-').with(value(Sign::Minus)),
        ))
    };
    (
        attempt(optional(integer_str(10)).skip(look_ahead(choice((
            char('.').skip(digit()),
            char('e'),
            char('E'),
        )))))
        .map(Option::unwrap_or_default),
        optional(char('.').with(integer_str(10))).map(Option::unwrap_or_default),
        optional(
            choice([char('e'), char('E')])
                .skip(skip_many(char('_')))
                .with((
                    optional(sign()).map(|sign| sign.unwrap_or(Sign::Plus)),
                    integer_str_allow_underscore(10),
                )),
        )
        .map(|sign_exp| sign_exp.unwrap_or((Sign::Plus, keyword!("")))),
    )
        .skip(not_followed_by(alpha_num()))
        .map(|(whole, decimal, (exp_sign, exp))| FloatSrc {
            whole,
            decimal,
            exp_sign,
            exp,
        })
}
pub(super) fn float<I>() -> impl Parser<I, Output = f64>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    float_src()
        .and_then(|src| {
            src.parse()
                .ok_or_else(|| <StreamErrorFor<I>>::message_static_message("magnitude overflow"))
        })
        .expected("float")
}
#[cfg(test)]
mod test {
    use crate::number::{float, integer_u64};
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
    #[test]
    fn not_float() {
        let src = "01234";
        assert!(float().easy_parse(src).is_err());
    }
    #[test]
    fn float_0() {
        let src = "1.2";
        let expected = 1.2;
        let (result, _) = float().easy_parse(src).unwrap();
        assert!((expected - result).abs() <= <f64>::EPSILON);
    }
    #[test]
    fn float_1() {
        let src = "01.2e3";
        let expected = 1.2e3;
        let (result, _) = float().easy_parse(src).unwrap();
        assert!((expected - result).abs() <= <f64>::EPSILON);
    }
    #[test]
    fn float_2() {
        let src = "01.2e+3";
        let expected = 1.2e+3;
        let (result, _) = float().easy_parse(src).unwrap();
        assert!((expected - result).abs() <= <f64>::EPSILON);
    }
    #[test]
    fn float_3() {
        let src = "01.2e-3";
        let expected = 1.2e-3;
        let (result, _) = float().easy_parse(src).unwrap();
        assert!((expected - result).abs() <= <f64>::EPSILON);
    }
}
