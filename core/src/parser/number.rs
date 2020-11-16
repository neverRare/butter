use crate::parser::error::ErrorType;
use crate::parser::node_type::Num;
use crate::parser::Error;

// TODO: this is copied from old code with few changes. add tests and refactor

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
    fn to_num(self) -> u32 {
        match self {
            Self::Hex => 16,
            Self::Dec => 10,
            Self::Oct => 8,
            Self::Bin => 2,
        }
    }
    fn is_valid(self, ch: char) -> bool {
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
struct DecimalPart {
    decimal: String,
    mantissa: String,
    mantissa_sign: Sign,
}
struct RegularNumber {
    whole: String,
    decimal: Option<DecimalPart>,
}
impl RegularNumber {
    fn parse(src: &str) -> Result<Self, Vec<Error>> {
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
        let mut errors = vec![];
        for (i, ch) in src.char_indices() {
            let ch_len = ch.len_utf8();
            let err = if let '_' = ch {
                continue;
            } else if let '.' = ch {
                tries_float = true;
                match mode {
                    Mode::Decimal => ErrorType::DoubleDecimal,
                    Mode::Mantissa(_) => ErrorType::DecimalOnMantissa,
                    Mode::Whole => {
                        mode = Mode::Decimal;
                        continue;
                    }
                }
            } else if let (Mode::Mantissa(true), Some(sign)) = (mode, Sign::from_char(ch)) {
                mantissa_sign = sign;
                mode = Mode::Mantissa(false);
                continue;
            } else {
                match ch {
                    '0'..='9' => {
                        match mode {
                            Mode::Whole => {
                                if errors.is_empty() {
                                    whole.push(ch)
                                }
                            }
                            Mode::Decimal => {
                                if errors.is_empty() {
                                    decimal.push(ch)
                                }
                            }
                            Mode::Mantissa(_) => {
                                mode = Mode::Mantissa(false);
                                if errors.is_empty() {
                                    mantissa.push(ch)
                                }
                            }
                        }
                        continue;
                    }
                    'e' | 'E' => {
                        tries_float = true;
                        if let Mode::Mantissa(_) = mode {
                            ErrorType::DoubleMantissa
                        } else {
                            mode = Mode::Mantissa(true);
                            continue;
                        }
                    }
                    _ => ErrorType::InvalidDigit(Radix::Dec),
                }
            };
            errors.push(Error {
                span: &src[i..i + ch_len],
                error: err,
            });
        }
        if errors.is_empty() {
            let decimal = if tries_float {
                Some(DecimalPart {
                    decimal,
                    mantissa,
                    mantissa_sign,
                })
            } else {
                None
            };
            Ok(RegularNumber {
                whole: whole.trim_start_matches('0').to_string(),
                decimal,
            })
        } else {
            Err(errors)
        }
    }
    fn to_num(&self) -> Option<Num> {
        let RegularNumber { whole, decimal } = self;
        let (decimal, mantissa, mantissa_sign) = match decimal {
            Some(decimal) => (&decimal.decimal, &decimal.mantissa, &decimal.mantissa_sign),
            None => {
                return match whole.parse::<u64>() {
                    Ok(num) => Some(Num::UInt(num)),
                    Err(_) => None,
                }
            }
        };
        let abscissa = whole.to_string() + &decimal;
        if abscissa.is_empty() {
            return Some(Num::UInt(0));
        }
        let whole_mantissa = if mantissa.is_empty() {
            -(decimal.len() as i128)
        } else {
            match mantissa.parse::<u32>() {
                Ok(mantissa) => {
                    mantissa_sign.to_num() as i128 * mantissa as i128 - decimal.len() as i128
                }
                Err(_) => {
                    return Some(Num::Float(match mantissa_sign {
                        Sign::Plus => f64::MAX,
                        Sign::Minus => f64::MIN_POSITIVE,
                    }))
                }
            }
        };
        let mantissa = abscissa.len() as i128 - 1 + whole_mantissa;
        if mantissa < f64::MIN_10_EXP as i128 {
            Some(Num::Float(f64::MIN_POSITIVE))
        } else if mantissa > f64::MAX_10_EXP as i128 {
            Some(Num::Float(f64::MAX))
        } else if whole_mantissa >= 0 {
            let whole = abscissa + &"0".repeat(whole_mantissa as usize);
            match whole.parse() {
                Ok(val) => Some(Num::UInt(val)),
                Err(_) => None,
            }
        } else {
            let abscissa = abscissa[0..1].to_string() + "." + &abscissa[1..];
            Some(Num::Float(
                abscissa.parse::<f64>().unwrap() * 10_f64.powi(mantissa as i32),
            ))
        }
    }
}
pub(super) fn parse_number(src: &str) -> Result<Num, Vec<Error>> {
    if let (Some("0"), Some(radix)) = (src.get(..1), src.get(1..2).and_then(Radix::from_str)) {
        let mut code = String::new();
        let mut invalid = vec![];
        for (i, ch) in src[2..].char_indices() {
            let err = if let '_' = ch {
                continue;
            } else if ch.is_alphanumeric() {
                if radix.is_valid(ch) {
                    code.push(ch);
                    continue;
                } else {
                    ErrorType::InvalidDigit(radix)
                }
            } else if let '.' = ch {
                ErrorType::DecimalOnInteger
            } else {
                unreachable!();
            };
            invalid.push(Error {
                span: &src[i + 2..i + ch.len_utf8() + 2],
                error: err,
            });
        }
        if invalid.is_empty() {
            match u64::from_str_radix(&code, radix.to_num()) {
                Ok(val) => Ok(Num::UInt(val)),
                Err(_) => Err(vec![Error {
                    span: src,
                    error: ErrorType::IntegerOverflow,
                }]),
            }
        } else {
            Err(invalid)
        }
    } else {
        let num = RegularNumber::parse(src);
        num.and_then(|num| match num.to_num() {
            Some(val) => Ok(val),
            None => Err(vec![Error {
                span: src,
                error: ErrorType::IntegerOverflow,
            }]),
        })
    }
}
