use crate::lexer::Num;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Radix {
    Hex,
    Dec,
    Oct,
    Bin,
}
impl Radix {
    fn from_str(src: &str) -> Option<Self> {
        match src {
            "x" | "X" => Some(Self::Hex),
            "o" | "O" => Some(Self::Oct),
            "b" | "B" => Some(Self::Bin),
            _ => None,
        }
    }
    fn to_num(&self) -> u32 {
        match self {
            Self::Hex => 16,
            Self::Dec => 10,
            Self::Oct => 8,
            Self::Bin => 2,
        }
    }
    fn is_valid(&self, ch: char) -> bool {
        match self {
            Self::Hex => matches!(ch, '0'..='9' | 'a'..='f' | 'A'..='F'),
            Self::Dec => matches!(ch, '0'..='9'),
            Self::Oct => matches!(ch, '0'..='7'),
            Self::Bin => matches!(ch, '0' | '1'),
        }
    }
}
enum Sign {
    Plus,
    Minus,
}
impl Sign {
    fn from_char(ch: char) -> Option<Self> {
        match ch {
            '+' => Some(Self::Plus),
            '-' => Some(Self::Minus),
            _ => None,
        }
    }
    fn to_num(&self) -> i32 {
        match self {
            Self::Plus => 1,
            Self::Minus => -1,
        }
    }
}
#[derive(PartialEq, Eq, Debug)]
pub enum InvalidChar {
    Invalid(Radix),
    DoubleDecimal,
    DoubleMantissa,
    DecimalOnMantissa,
    DecimalOnInteger,
}
#[derive(PartialEq, Eq, Debug)]
pub enum NumError<'a> {
    InvalidChar(Vec<(&'a str, InvalidChar)>),
    Overflow,
}
struct RegularNumber {
    whole: String,
    decimal: String,
    mantissa: String,
    mantissa_sign: Sign,
    tries_float: bool,
}
impl RegularNumber {
    fn parse(src: &str) -> (usize, Result<Self, NumError>) {
        #[derive(Clone, Copy)]
        enum Mode {
            Whole,
            Decimal,
            Mantissa(bool),
        }
        let mut mode = Mode::Whole;
        let mut whole = String::new();
        let mut decimal = String::new();
        let mut mantissa = String::new();
        let mut mantissa_sign = Sign::Plus;
        let mut tries_float = false;
        let mut len = src.len();
        let mut invalid = vec![];
        for (i, ch) in src.char_indices() {
            let ch_len = ch.len_utf8();
            let err = if let '_' = ch {
                continue;
            } else if let ('.', Some('0'..='9')) = (ch, src[i + ch_len..].chars().next()) {
                match mode {
                    Mode::Decimal => InvalidChar::DoubleDecimal,
                    Mode::Mantissa(_) => InvalidChar::DecimalOnMantissa,
                    Mode::Whole => {
                        mode = Mode::Decimal;
                        continue;
                    }
                }
            } else if let (Mode::Mantissa(true), Some(sign)) = (mode, Sign::from_char(ch)) {
                mantissa_sign = sign;
                mode = Mode::Mantissa(false);
                continue;
            } else if ch.is_alphanumeric() {
                match ch {
                    '0'..='9' => {
                        match mode {
                            Mode::Whole => whole.push(ch),
                            Mode::Decimal => decimal.push(ch),
                            Mode::Mantissa(_) => {
                                tries_float = true;
                                mode = Mode::Mantissa(false);
                                mantissa.push(ch);
                            }
                        }
                        continue;
                    }
                    'e' | 'E' => {
                        if let Mode::Mantissa(_) = mode {
                            InvalidChar::DoubleMantissa
                        } else {
                            mode = Mode::Mantissa(true);
                            continue;
                        }
                    }
                    _ => InvalidChar::Invalid(Radix::Dec),
                }
            } else {
                len = i;
                break;
            };
            invalid.push((&src[i..i + ch_len], err));
        }
        let val = if invalid.is_empty() {
            Ok(RegularNumber {
                whole: whole.trim_start_matches('0').to_string(),
                decimal: decimal.trim_end_matches('0').to_string(),
                mantissa,
                mantissa_sign,
                tries_float,
            })
        } else {
            Err(NumError::InvalidChar(invalid))
        };
        (len, val)
    }
    fn to_num(&self) -> Option<Num> {
        let RegularNumber {
            whole,
            decimal,
            mantissa,
            mantissa_sign,
            tries_float,
        } = self;
        let absissa = whole.to_string() + decimal;
        if absissa.is_empty() {
            return Some(Num::UInt(0));
        }
        let whole_mantissa = if mantissa.is_empty() {
            -(decimal.len() as i64)
        } else {
            match mantissa.parse::<u32>() {
                Ok(mantissa) => {
                    mantissa_sign.to_num() as i64 * mantissa as i64 - decimal.len() as i64
                }
                Err(_) => {
                    return if *tries_float {
                        Some(Num::Float(match mantissa_sign {
                            Sign::Plus => f64::MAX,
                            Sign::Minus => f64::MIN_POSITIVE,
                        }))
                    } else {
                        None
                    }
                }
            }
        };
        let mantissa = absissa.len() as i64 - 1 + whole_mantissa;
        if mantissa < f64::MIN_10_EXP as i64 {
            if *tries_float {
                Some(Num::Float(f64::MIN_POSITIVE))
            } else {
                None
            }
        } else if mantissa > f64::MAX_10_EXP as i64 {
            if *tries_float {
                Some(Num::Float(f64::MAX))
            } else {
                None
            }
        } else if whole_mantissa >= 0 {
            let whole = absissa + &"0".repeat(whole_mantissa as usize);
            match whole.parse() {
                Ok(val) => Some(Num::UInt(val)),
                Err(_) if *tries_float => Some(Num::Float(whole.parse().unwrap())),
                Err(_) => None,
            }
        } else {
            let absissa = absissa[0..1].to_string() + "." + &absissa[1..];
            Some(Num::Float(
                absissa.parse::<f64>().unwrap() * 10_f64.powi(mantissa as i32),
            ))
        }
    }
}
pub fn parse_number(src: &str) -> (usize, Result<Num, NumError>) {
    if let (Some("0"), Some(radix)) = (src.get(..1), src.get(1..2).and_then(Radix::from_str)) {
        let mut code = String::new();
        let mut len = src.len();
        let mut invalid = vec![];
        for (i, ch) in src[2..].char_indices() {
            let err = if let '_' = ch {
                continue;
            } else if ch.is_alphanumeric() {
                if radix.is_valid(ch) {
                    code.push(ch);
                    continue;
                } else {
                    InvalidChar::Invalid(radix)
                }
            } else if let ('.', Some('0'..='9')) = (ch, src[i + ch.len_utf8()..].chars().next()) {
                InvalidChar::DecimalOnInteger
            } else {
                len = i;
                break;
            };
            invalid.push((&src[i + 2..i + ch.len_utf8() + 2], err));
        }
        let len = len + 2;
        if invalid.is_empty() {
            match u64::from_str_radix(&code, radix.to_num()) {
                Ok(val) => (len, Ok(Num::UInt(val))),
                Err(_) => (len, Err(NumError::Overflow)),
            }
        } else {
            (len, Err(NumError::InvalidChar(invalid)))
        }
    } else {
        let (len, num) = RegularNumber::parse(src);
        let val = num.and_then(|num| match num.to_num() {
            Some(val) => Ok(val),
            None => Err(NumError::Overflow),
        });
        (len, val)
    }
}
