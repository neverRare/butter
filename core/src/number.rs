use crate::lexer::{LexerError, Num};

pub enum ParseResult {
    Ok(usize, Num),
    Err(LexerError, usize, usize),
    None,
}
pub fn parse_number(src: &str) -> ParseResult {
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
                        return ParseResult::Err(LexerError::InvalidNumber, i, i + ch.len_utf8());
                    }
                    code.push(ch);
                } else {
                    len = i;
                    break;
                }
            }
            return match u64::from_str_radix(&code, radix) {
                Ok(val) => ParseResult::Ok(len + 2, Num::UInt(val)),
                Err(_) => ParseResult::Err(LexerError::Overflow, 0, len + 2),
            };
        }
    }
    todo!()
}
