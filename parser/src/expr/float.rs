use crate::expr::integer::{integer_str, integer_str_allow_underscore, parse_digit, parse_i32};
use combine::{
    attempt, choice,
    error::StreamError,
    look_ahead, not_followed_by, optional,
    parser::{
        char::{alpha_num, char, digit},
        range::take_while,
    },
    stream::StreamErrorFor,
    ParseError, Parser, RangeStream,
};

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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct FloatSrc<'a> {
    whole: &'a str,
    decimal: &'a str,
    exp_sign: Sign,
    exp: &'a str,
}
impl<'a> FloatSrc<'a> {
    fn parse(self) -> Option<f64> {
        // TODO: avoid string allocation, precision must be kept
        let src: String = format!(
            "0{}.{}0e{}0{}",
            self.whole,
            self.decimal,
            self.exp_sign.into_char(),
            self.exp,
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
        )))))
        .map(Option::unwrap_or_default),
        optional(char('.').with(integer_str(10))).map(Option::unwrap_or_default),
        optional(
            choice([char('e'), char('E')])
                .skip(take_while(|ch: char| ch == '_'))
                .with((
                    optional(sign()).map(|sign| sign.unwrap_or(Sign::Plus)),
                    integer_str_allow_underscore(10),
                )),
        )
        .map(|sign_exp| sign_exp.unwrap_or((Sign::Plus, ""))),
    )
        .skip(not_followed_by(alpha_num()))
        .map(|(whole, decimal, (exp_sign, exp))| FloatSrc {
            whole,
            decimal,
            exp_sign,
            exp,
        })
}
combine::parser! {
    pub(crate) fn float['a, I]()(I) -> f64
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        float_src()
            .and_then(|src| match src.parse() {
                Some(float) => Ok(float),
                None => Err(<StreamErrorFor<I>>::message_static_message("magnitude overflow"))
            })
            .expected("float")
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
