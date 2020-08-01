use crate::lexer::LexerError;

struct ParseBytes<'a> {
    quote: char,
    src: &'a str,
    byte_src: &'a [u8],
    i: usize,
    char_i: usize,
}
impl<'a> ParseBytes<'a> {
    fn new(quote: char, src: &'a str) -> Self {
        Self {
            quote,
            src,
            byte_src: src.as_bytes(),
            i: 0,
            char_i: 0,
        }
    }
}
enum ParseResult {
    Yield(u8),
    Noop,
    Done(usize),
    Error(LexerError, usize, usize),
}
impl<'a> Iterator for ParseBytes<'a> {
    type Item = ParseResult;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.char_i {
            let src = &self.src[self.i..];
            match src.chars().next() {
                Some(quote) if quote == self.quote => Some(ParseResult::Done(self.i)),
                Some('\\') => {
                    let (len, byte) = match src[1..].chars().next() {
                        Some(code) if code == 'x' || code == 'X' => {
                            let result = match src.get(2..4) {
                                Some(code) => u8::from_str_radix(code, 16).map_err(|_| self.i + 4),
                                None => Err(self.i + 2),
                            };
                            match result {
                                Ok(val) => (4, val),
                                Err(res) => {
                                    return Some(ParseResult::Error(
                                        LexerError::InvalidEscape,
                                        self.i,
                                        res,
                                    ))
                                }
                            }
                        }
                        Some(code) => {
                            let byte = match code {
                                '\\' => b'\\',
                                '\'' => b'\'',
                                '"' => b'"',
                                'n' => b'\n',
                                'r' => b'\r',
                                't' => b'\t',
                                'v' => b'\x30',
                                '0' => b'\0',
                                _ => {
                                    return Some(ParseResult::Error(
                                        LexerError::InvalidEscape,
                                        self.i,
                                        self.i + 2,
                                    ))
                                }
                            };
                            (2, byte)
                        }
                        None => {
                            return Some(ParseResult::Error(
                                LexerError::UnterminatedQuote,
                                0,
                                self.src.len(),
                            ))
                        }
                    };
                    self.i += len;
                    self.char_i += len;
                    Some(ParseResult::Yield(byte))
                }
                Some(val) => {
                    self.char_i += val.len_utf8();
                    Some(ParseResult::Noop)
                }
                None => Some(ParseResult::Error(
                    LexerError::UnterminatedQuote,
                    0,
                    self.src.len(),
                )),
            }
        } else {
            let i = self.i;
            self.i += 1;
            Some(ParseResult::Yield(self.byte_src[i]))
        }
    }
}
pub fn parse_string(
    quote: char,
    rest: &str,
) -> Result<(usize, Vec<u8>), (LexerError, usize, usize)> {
    let mut vec = vec![];
    for res in ParseBytes::new(quote, rest) {
        match res {
            ParseResult::Yield(val) => vec.push(val),
            ParseResult::Error(err, from, to) => return Err((err, from, to)),
            ParseResult::Done(len) => return Ok((len, vec)),
            ParseResult::Noop => (),
        }
    }
    unreachable!()
}
