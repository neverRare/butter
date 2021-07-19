use crate::expr::integer::integer_str;
use crate::expr::integer::integer_str_allow_underscore;
use crate::expr::integer::parse_digit;
use crate::expr::integer::parse_i32;
use combine::attempt;
use combine::choice;
use combine::error::StreamError;
use combine::look_ahead;
use combine::not_followed_by;
use combine::optional;
use combine::parser;
use combine::parser::char::alpha_num;
use combine::parser::char::char;
use combine::parser::char::digit;
use combine::parser::range::take_while;
use combine::stream::StreamErrorFor;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;
use std::convert::TryInto;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Sign {
    Plus,
    Minus,
}
impl Sign {
    fn into_i32(self) -> i32 {
        match self {
            Self::Plus => 1,
            Self::Minus => -1,
        }
    }
}
impl Default for Sign {
    fn default() -> Self {
        Self::Plus
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct FloatSrc<'a> {
    whole: &'a str,
    decimal: &'a str,
    exp_sign: Sign,
    exp: &'a str,
}
impl<'a> FloatSrc<'a> {
    fn parse(self) -> Option<f64> {
        let whole = self.whole.trim_start_matches(&['_', '0'] as &[_]);
        let digit_count: Option<i32> = self
            .whole
            .chars()
            .filter(|ch| *ch != '_')
            .count()
            .try_into()
            .ok();
        let exp = match (digit_count, parse_i32(self.exp, 10)) {
            (Some(digit_count), Some(exp)) => exp
                .checked_mul(self.exp_sign.into_i32())
                .and_then(|exp| exp.checked_add(digit_count - 1)),
            _ => None,
        };
        let exp = match exp {
            Some(exp) if exp <= <f64>::MAX_10_EXP => exp,
            _ => return None,
        };
        let mantissa_iter = whole
            .chars()
            .chain(self.decimal.chars())
            .filter(|ch| *ch != '_')
            .take(<f64>::DIGITS as usize);
        let mut result = 0.;
        for (digit, place) in mantissa_iter.zip(1..) {
            let magnitude = exp - place;
            let digit = parse_digit(digit, 10).unwrap() as f64;
            result += digit * 10_f64.powi(magnitude);
        }
        assert!(result.is_finite());
        Some(result)
    }
}
fn float_src<'a, I>() -> impl Parser<I, Output = FloatSrc<'a>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let sign = || {
        choice((
            char('+').map(|_| Sign::Plus),
            char('-').map(|_| Sign::Minus),
        ))
    };
    (
        attempt(optional(integer_str(10)).skip(look_ahead(choice((
            char('.').skip(digit()),
            char('e'),
            char('E'),
        ))))),
        optional(char('.').with(integer_str(10))),
        optional(
            choice([char('e'), char('E')])
                .skip(take_while(|ch: char| ch == '_'))
                .with((optional(sign()), integer_str_allow_underscore(10))),
        ),
    )
        .skip(not_followed_by(alpha_num()))
        .map(|(whole, decimal, exp)| {
            let (exp_sign, exp) = exp.unwrap_or_else(Default::default);
            FloatSrc {
                whole: whole.unwrap_or_else(Default::default),
                decimal: decimal.unwrap_or_else(Default::default),
                exp_sign: exp_sign.unwrap_or_else(Default::default),
                exp,
            }
        })
}
parser! {
    pub fn float['a, I]()(I) -> f64
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        float_src().and_then(|src| match src.parse() {
            Some(float) => Ok(float),
            None => Err(<StreamErrorFor<I>>::message_static_message("magnitude overflow"))
        })
    }
}
#[cfg(test)]
mod test {
    use crate::expr::float::float;
    use combine::EasyParser;

    #[test]
    fn not_float() {
        let src = "01234";
        assert!(float().easy_parse(src).is_err());
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
