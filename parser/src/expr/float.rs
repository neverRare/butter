use crate::expr::integer::{integer_str, integer_str_allow_underscore};
use combine::{
    attempt, choice,
    error::StreamError,
    look_ahead, not_followed_by, optional,
    parser::char::{alpha_num, char, digit},
    skip_many,
    stream::StreamErrorFor,
    ParseError, Parser, Stream,
};
use string_cache::DefaultAtom;

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
    whole: DefaultAtom,
    decimal: DefaultAtom,
    exp_sign: Sign,
    exp: DefaultAtom,
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
                .skip(skip_many(char('_')))
                .with((
                    optional(sign()).map(|sign| sign.unwrap_or(Sign::Plus)),
                    integer_str_allow_underscore(10),
                )),
        )
        .map(|sign_exp| sign_exp.unwrap_or((Sign::Plus, DefaultAtom::from("")))),
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
    pub(crate) fn float[I]()(I) -> f64
    where [
        I: Stream<Token = char>,
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
