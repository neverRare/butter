use crate::lexer::Float;
use crate::lexer::Sign;
use crate::parser::integer::parse_i32;

pub fn parse_float(float: Float) -> Option<f64> {
    let exp_sign = match float.exp_sign {
        Sign::Plus => 1_i32,
        Sign::Minus => -1,
    };
    let exp = exp_sign.checked_mul(parse_i32(10, float.exp.as_bytes())?)?;
    if f64::MIN_10_EXP <= exp && exp <= f64::MAX_10_EXP {
        let whole = float
            .whole
            .as_bytes()
            .iter()
            .filter(|ch| **ch != b'_')
            .rev()
            .map(|ch| *ch as u32 - b'0' as u32)
            .zip((exp..).map(|exp| 10_f64.powi(exp)))
            .map(|(digit, place)| place * digit as f64)
            .fold(0.00, |left, right| left + right);
        let decimal = float
            .decimal
            .as_bytes()
            .iter()
            .filter(|ch| **ch != b'_')
            .map(|ch| *ch as u32 - b'0' as u32)
            .zip((1..).map(|i| 10_f64.powi(exp - i)))
            .map(|(digit, place)| place * digit as f64)
            .fold(0.00, |left, right| left + right);
        Some(whole + decimal)
    } else {
        None
    }
}
#[cfg(test)]
mod test {
    use crate::lexer::Float;
    use crate::lexer::Sign;
    use crate::parser::float::parse_float;
    use util::compare_float;

    #[test]
    fn parse() {
        let float = parse_float(Float {
            whole: "1",
            decimal: "2",
            exp_sign: Sign::Plus,
            exp: "3",
        })
        .unwrap();
        assert!(compare_float(float, 1.2e3));
        let float = parse_float(Float {
            whole: "1",
            decimal: "2",
            exp_sign: Sign::Minus,
            exp: "3",
        })
        .unwrap();
        assert!(compare_float(float, 1.2e-3));
        let float = parse_float(Float {
            whole: "",
            decimal: "2",
            exp_sign: Sign::Plus,
            exp: "3",
        })
        .unwrap();
        assert!(compare_float(float, 0.2e3));
        let float = parse_float(Float {
            whole: "1",
            decimal: "",
            exp_sign: Sign::Plus,
            exp: "3",
        })
        .unwrap();
        assert!(compare_float(float, 1e3));
        let float = parse_float(Float {
            whole: "1",
            decimal: "2",
            exp_sign: Sign::Plus,
            exp: "",
        })
        .unwrap();
        assert!(compare_float(float, 1.2));
    }
}
