#[derive(PartialEq, Eq, Debug)]
pub enum EscapeError {
    InvalidChar,
    InvalidHexChar,
    HexCharTerminated,
}
#[derive(PartialEq, Eq, Debug)]
pub enum StrError<'a> {
    InvalidEscape(Vec<(&'a str, EscapeError)>),
    Unterminated,
}
pub fn parse_string(quote: char, rest: &str) -> (usize, Result<Vec<u8>, StrError>) {
    let mut vec: Vec<u8> = vec![];
    let mut i = 0;
    let mut errors: Vec<(&str, EscapeError)> = vec![];
    let result = loop {
        let first = match rest[i..].chars().next() {
            Some(ch) => ch,
            None => break Err(StrError::Unterminated),
        };
        if let '\\' = first {
            let escape = match rest[i + 1..].chars().next() {
                Some(ch) => ch,
                None => break Err(StrError::Unterminated),
            };
            if let 'x' = escape {
                let mut len = 0;
                let mut codes: Vec<u8> = vec![];
                let code_rest = &rest[i + 2..];
                let code_chars = code_rest
                    .char_indices()
                    .take(2)
                    .take_while(|(_, ch)| *ch != first && *ch != '\\');
                for (ind, ch) in code_chars {
                    len = ind + ch.len_utf8();
                    match ch.to_digit(16) {
                        Some(val) => codes.push(val as u8),
                        None => {
                            errors.push((
                                &code_rest[ind..ind + ch.len_utf8()],
                                EscapeError::InvalidHexChar,
                            ));
                        }
                    }
                }
                if codes.len() != 2 {
                    let slice = if codes.is_empty() {
                        &rest[i + 1..i + 2]
                    } else {
                        &code_rest[..len]
                    };
                    errors.push((slice, EscapeError::HexCharTerminated));
                } else if errors.is_empty() {
                    vec.push((codes[0] << 4) + codes[1]);
                }
                i += 2 + len;
            } else {
                let byte = match escape {
                    '\\' => b'\\',
                    '\'' => b'\'',
                    '"' => b'"',
                    'n' => b'\n',
                    'r' => b'\r',
                    't' => b'\t',
                    'v' => b'\x30',
                    '0' => b'\0',
                    _ => {
                        errors.push((
                            &rest[i + 1..i + 1 + escape.len_utf8()],
                            EscapeError::InvalidChar,
                        ));
                        i += 1 + escape.len_utf8();
                        continue;
                    }
                };
                i += 2;
                if errors.is_empty() {
                    vec.push(byte);
                }
            }
        } else if quote == first {
            break if errors.is_empty() {
                Ok(vec)
            } else {
                Err(StrError::InvalidEscape(errors))
            };
        } else {
            i += first.len_utf8();
            let mut string = String::with_capacity(first.len_utf8());
            string.push(first);
            vec.append(&mut string.into());
        }
    };
    (i, result)
}
