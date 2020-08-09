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
            let escape = match rest[i + first.len_utf8()..].chars().next() {
                Some(ch) => ch,
                None => break Err(StrError::Unterminated),
            };
            if let 'x' = escape {
                todo!();
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
