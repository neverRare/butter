struct ParseBytes<'a> {
    quote: char,
    src: &'a str,
    i: usize,
    char_i: usize,
}
impl<'a> ParseBytes<'a> {
    fn new(quote: char, src: &'a str) -> Self {
        Self {
            quote,
            src,
            i: 0,
            char_i: 0,
        }
    }
}
pub enum ParseError {
    InvalidEscape,
    UnterminatedQuote,
}
enum ParseResult {
    Yield(u8),
    Done(usize),
    Error(ParseError, usize, usize),
}
impl<'a> Iterator for ParseBytes<'a> {
    type Item = ParseResult;
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
pub fn parse_string(
    quote: char,
    rest: &str,
) -> Result<(usize, Vec<u8>), (ParseError, usize, usize)> {
    let mut vec = vec![];
    for res in ParseBytes::new(quote, rest) {
        match res {
            ParseResult::Yield(val) => vec.push(val),
            ParseResult::Error(err, from, to) => return Err((err, from, to)),
            ParseResult::Done(len) => return Ok((len, vec)),
        }
    }
    unreachable!()
}
