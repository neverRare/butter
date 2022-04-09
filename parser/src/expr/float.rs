use crate::expr::integer::{integer_str, integer_str_allow_underscore};
use combine::{
    attempt, choice,
    error::StreamError,
    look_ahead, not_followed_by, optional,
    parser::char::{alpha_num, char, digit},
    skip_many,
    stream::StreamErrorFor,
    value, ParseError, Parser, Stream,
};
use hir::Atom;

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
        .map(|sign_exp| sign_exp.unwrap_or((Sign::Plus, Atom::from("")))),
    )
        .skip(not_followed_by(alpha_num()))
        .map(|(whole, decimal, (exp_sign, exp))| FloatSrc {
            whole,
            decimal,
            exp_sign,
            exp,
        })
}
pub(crate) fn float<I>() -> impl Parser<I, Output = f64>
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
