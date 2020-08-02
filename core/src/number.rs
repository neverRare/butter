use crate::lexer::{LexerError, Num};

pub fn parse_number(src: &str) -> Result<(usize, Num), (LexerError, usize, usize)> {
    if let Some("0") = src.get(..1) {
        let radix = match src.get(1..2) {
            Some("x") | Some("X") => Some(16),
            Some("o") | Some("O") => Some(8),
            Some("b") | Some("B") => Some(2),
            _ => None,
        };
        if let Some(radix) = radix {
            let mut code = String::new();
            let mut len = 0;
            for (i, ch) in src[2..].char_indices() {
                if let '_' = ch {
                    continue;
                } else if ch.is_alphanumeric() {
                    let valid = match radix {
                        16 => matches!(ch, '0'..='9' | 'a'..='f' | 'A'..='F'),
                        8 => matches!(ch, '0'..='7'),
                        2 => matches!(ch, '0' | '1'),
                        _ => unreachable!(),
                    };
                    if !valid {
                        let err = match radix {
                            16 => LexerError::InvalidCharOnHex,
                            8 => LexerError::InvalidCharOnOct,
                            2 => LexerError::InvalidCharOnBin,
                            _ => unreachable!(),
                        };
                        return Err((err, i, i + ch.len_utf8()));
                    }
                    code.push(ch);
                } else {
                    len = i;
                    break;
                }
            }
            return match u64::from_str_radix(&code, radix) {
                Ok(val) => {
                    if let (Some("."), Some('0'..='9')) = (
                        src.get(len..len + 1),
                        src.get(len + 1..).and_then(|val| val.chars().next()),
                    ) {
                        Err((LexerError::DecimalOnInt, len, len + 1))
                    } else {
                        Ok((len + 2, Num::UInt(val)))
                    }
                }
                Err(_) => Err((LexerError::Overflow, 0, len + 2)),
            };
        }
    }
    enum Mode {
        Whole,
        Decimal,
        Magnitude(bool),
    }
    enum Sign {
        Plus,
        Minus,
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
        if let '_' = ch {
            continue;
        } else if let ('.', Some('0'..='9')) = (ch, src[i + ch_len..].chars().next()) {
            match mode {
                Mode::Decimal => return Err((LexerError::DoubleDecimal, i, i + ch_len)),
                Mode::Magnitude(_) => return Err((LexerError::DecimalOnMagnitude, i, i + ch_len)),
                Mode::Whole => {
                    mode = Mode::Decimal;
                    continue;
                }
            }
        } else if let Mode::Magnitude(true) = mode {
            let sign = match ch {
                '-' => Some(Sign::Minus),
                '+' => Some(Sign::Plus),
                _ => None,
            };
            if let Some(sign) = sign {
                magnitude_sign = sign;
                mode = Mode::Magnitude(false);
                continue;
            }
        }
        if ch.is_alphanumeric() {
            match ch {
                '0'..='9' => match mode {
                    Mode::Whole => whole.push(ch),
                    Mode::Decimal => decimal.push(ch),
                    Mode::Magnitude(_) => {
                        tries_float = true;
                        mode = Mode::Magnitude(false);
                        magnitude.push(ch);
                    }
                },
                'e' | 'E' => {
                    if let Mode::Magnitude(_) = mode {
                        return Err((LexerError::DoubleMagnitude, i, i + ch_len));
                    } else {
                        mode = Mode::Magnitude(true);
                    }
                }
                _ => return Err((LexerError::InvalidCharOnNum, i, i + ch_len)),
            }
        } else {
            len = i;
            break;
        }
    }
    let whole = whole.trim_start_matches('0');
    let decimal = decimal.trim_end_matches('0');
    let whole = whole.to_string() + decimal;
    if whole.is_empty() {
        return Ok((len, Num::UInt(0)));
    }
    let places = if magnitude.is_empty() {
        -(decimal.len() as i64)
    } else {
        match magnitude.parse::<i64>() {
            Ok(magnitude) => {
                let sign = match magnitude_sign {
                    Sign::Plus => 1,
                    Sign::Minus => -1,
                };
                sign * magnitude - (decimal.len() as i64)
            }
            Err(_) => return Err((LexerError::MagnitudeOverflow, 0, len)),
        }
    };
    let whole_places = whole.len() as i64 - 1 + places;
    if whole_places < i32::MIN as i64 || whole_places > i32::MAX as i64 {
        Err((LexerError::MagnitudeOverflow, 0, len))
    } else if places >= 0 {
        let mut whole = whole;
        whole.push_str(&"0".repeat(places as usize));
        match whole.parse::<u64>() {
            Ok(val) => Ok((len, Num::UInt(val))),
            Err(_) => {
                if tries_float {
                    Ok((len, Num::Float(whole.parse().unwrap())))
                } else {
                    Err((LexerError::Overflow, 0, len))
                }
            }
        }
    } else {
        let mut val = 0f64;
        let mut mag = whole_places;
        for ch in whole.chars() {
            let digit = match ch {
                '0' => {
                    mag -= 1;
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
            val += digit * 10f64.powf(mag as f64);
            mag -= 1;
        }
        Ok((len, Num::Float(val)))
    }
}
