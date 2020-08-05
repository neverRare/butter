use crate::lexer::{LexerError, Num};
use crate::span::Span;

enum Radix {
    Hex,
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
    fn to_radix(&self) -> u32 {
        match self {
            Self::Hex => 16,
            Self::Oct => 8,
            Self::Bin => 2,
        }
    }
    fn invalid_digit_err(&self, ch: char) -> Option<LexerError> {
        let valid = match self {
            Self::Hex => matches!(ch, '0'..='9' | 'a'..='f' | 'A'..='F'),
            Self::Oct => matches!(ch, '0'..='7'),
            Self::Bin => matches!(ch, '0' | '1'),
        };
        if valid {
            None
        } else {
            Some(match self {
                Self::Hex => LexerError::InvalidCharOnHex,
                Self::Oct => LexerError::InvalidCharOnOct,
                Self::Bin => LexerError::InvalidCharOnBin,
            })
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
struct RegularNumber {
    whole: String,
    decimal: String,
    magnitude: String,
    magnitude_sign: Sign,
    tries_float: bool,
}
impl RegularNumber {
    fn parse(src: &str) -> Result<(usize, Self), Span<LexerError>> {
        #[derive(Clone, Copy)]
        enum Mode {
            Whole,
            Decimal,
            Magnitude(bool),
        }
        let mut mode = Mode::Whole;
        let mut whole = String::new();
        let mut decimal = String::new();
        let mut magnitude = String::new();
        let mut magnitude_sign = Sign::Plus;
        let mut tries_float = false;
        let mut len = src.len();
        for (i, ch) in src.char_indices() {
            let ch_len = ch.len_utf8();
            let err = if let '_' = ch {
                continue;
            } else if let ('.', Some('0'..='9')) = (ch, src[i + ch_len..].chars().next()) {
                match mode {
                    Mode::Decimal => LexerError::DoubleDecimal,
                    Mode::Magnitude(_) => LexerError::DecimalOnMagnitude,
                    Mode::Whole => {
                        mode = Mode::Decimal;
                        continue;
                    }
                }
            } else if let (Mode::Magnitude(true), Some(sign)) = (mode, Sign::from_char(ch)) {
                magnitude_sign = sign;
                mode = Mode::Magnitude(false);
                continue;
            } else if ch.is_alphanumeric() {
                match ch {
                    '0'..='9' => {
                        match mode {
                            Mode::Whole => whole.push(ch),
                            Mode::Decimal => decimal.push(ch),
                            Mode::Magnitude(_) => {
                                tries_float = true;
                                mode = Mode::Magnitude(false);
                                magnitude.push(ch);
                            }
                        }
                        continue;
                    }
                    'e' | 'E' => {
                        if let Mode::Magnitude(_) = mode {
                            LexerError::DoubleMagnitude
                        } else {
                            mode = Mode::Magnitude(true);
                            continue;
                        }
                    }
                    _ => LexerError::InvalidCharOnNum,
                }
            } else {
                len = i;
                break;
            };
            return Err(Span::new(src, err, i..i + ch_len));
        }
        Ok((
            len,
            RegularNumber {
                whole,
                decimal,
                magnitude,
                magnitude_sign,
                tries_float,
            },
        ))
    }
    fn to_num(&self) -> Result<Num, LexerError> {
        let RegularNumber {
            whole,
            decimal,
            magnitude,
            magnitude_sign,
            tries_float,
        } = self;
        let whole = whole.trim_start_matches('0');
        let decimal = decimal.trim_end_matches('0');
        let absissa = whole.to_string() + decimal;
        if absissa.is_empty() {
            return Ok(Num::UInt(0));
        }
        let whole_magnitude = if magnitude.is_empty() {
            -(decimal.len() as i64)
        } else {
            match magnitude.parse::<i64>() {
                Ok(magnitude) => magnitude_sign.to_num() as i64 * magnitude - decimal.len() as i64,
                Err(_) => return Err(LexerError::MagnitudeOverflow),
            }
        };
        let magnitude = absissa.len() as i64 - 1 + whole_magnitude;
        if magnitude < i32::MIN as i64 || magnitude > i32::MAX as i64 {
            Err(LexerError::MagnitudeOverflow)
        } else if whole_magnitude >= 0 {
            let mut whole = absissa;
            whole.push_str(&"0".repeat(whole_magnitude as usize));
            match whole.parse::<u64>() {
                Ok(val) => Ok(Num::UInt(val)),
                Err(_) if *tries_float => Ok(Num::Float(whole.parse().unwrap())),
                Err(_) => Err(LexerError::IntegerOverflow),
            }
        } else {
            let mut val = 0f64;
            let mut magnitude = magnitude;
            for ch in absissa.chars() {
                let digit = match ch {
                    '0' => {
                        magnitude -= 1;
                        continue;
                    }
                    '1' => 1f64,
                    '2' => 2f64,
                    '3' => 3f64,
                    '4' => 4f64,
                    '5' => 5f64,
                    '6' => 6f64,
                    '7' => 7f64,
                    '8' => 8f64,
                    '9' => 9f64,
                    _ => unreachable!(),
                };
                val += digit * 10f64.powf(magnitude as f64);
                magnitude -= 1;
            }
            Ok(Num::Float(val))
        }
    }
}
pub fn parse_number(src: &str) -> Result<(usize, Num), Span<LexerError>> {
    if let (Some("0"), Some(radix)) = (src.get(..1), src.get(1..2).and_then(Radix::from_str)) {
        let mut code = String::new();
        let mut len = 0;
        for (i, ch) in src[2..].char_indices() {
            if let '_' = ch {
                continue;
            } else if ch.is_alphanumeric() {
                if let Some(err) = radix.invalid_digit_err(ch) {
                    return Err(Span::new(src, err, i..i + ch.len_utf8()));
                } else {
                    code.push(ch);
                }
            } else {
                len = i;
                break;
            }
        }
        match u64::from_str_radix(&code, radix.to_radix()) {
            Ok(val) => {
                if let (Some("."), Some('0'..='9')) = (
                    src.get(len..len + 1),
                    src.get(len + 1..).and_then(|val| val.chars().next()),
                ) {
                    Err(Span::new(src, LexerError::DecimalOnInt, len..len + 1))
                } else {
                    Ok((len + 2, Num::UInt(val)))
                }
            }
            Err(_) => Err(Span::new(src, LexerError::IntegerOverflow, 0..len + 2)),
        }
    } else {
        let (len, num) = RegularNumber::parse(src)?;
        match num.to_num() {
            Ok(num) => Ok((len, num)),
            Err(err) => Err(Span::new(src, err, 0..len)),
        }
    }
}
