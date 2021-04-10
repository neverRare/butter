use crate::parser::expr::integer::parse_digit;
use combine::between;
use combine::choice;
use combine::error::StreamError;
use combine::many;
use combine::parser;
use combine::parser::char::char;
use combine::parser::char::hex_digit;
use combine::satisfy;
use combine::stream::StreamErrorFor;
use combine::ParseError;
use combine::Parser;
use combine::RangeStream;
use std::iter::repeat;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Char {
    Char(char),
    Byte(u8),
}
fn char_inside<'a, I>(delimiter: char) -> impl Parser<I, Output = Char>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let simple_escape = || {
        choice((
            char('\\').map(|_| b'\\'),
            char('"').map(|_| b'"'),
            char('\'').map(|_| b'\''),
            char('n').map(|_| b'\n'),
            char('r').map(|_| b'\r'),
            char('t').map(|_| b'\t'),
            char('v').map(|_| b'\t'),
            char('0').map(|_| 0),
        ))
    };
    let byte = || {
        (hex_digit(), hex_digit()).map(|(sixteens, ones)| {
            parse_digit(sixteens, 16).unwrap() * 16 + parse_digit(ones, 16).unwrap()
        })
    };
    choice((
        char('\\')
            .with(choice((simple_escape(), char('x').with(byte()))))
            .map(Char::Byte),
        satisfy(move |ch: char| ch != delimiter && ch != '\n').map(Char::Char),
    ))
}
parser! {
    pub fn char_literal['a, I]()(I) -> u8
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        between(char('\''), char('\''), char_inside('\'')).and_then(|ch| match ch {
            Char::Byte(byte) => Ok(byte),
            Char::Char(ch) => {
                if ch.len_utf8() == 1 {
                    Ok(ch as u8)
                } else {
                    Err(<StreamErrorFor<I>>::message_static_message(
                        "multiple character in char literal",
                    ))
                }
            }
        })
    }
}
#[derive(Default, Clone, PartialEq, Eq, Debug)]
struct StringLiteral(Vec<u8>);
impl Extend<Char> for StringLiteral {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = Char>,
    {
        let Self(vec) = self;
        for ch in iter {
            match ch {
                Char::Byte(byte) => vec.push(byte),
                Char::Char(ch) => {
                    let len_before = vec.len();
                    vec.extend(repeat(0).take(ch.len_utf8()));
                    let end = &mut vec[len_before..];
                    ch.encode_utf8(end);
                }
            }
        }
    }
}
pub fn string_literal<'a, I>() -> impl Parser<I, Output = Vec<u8>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    between(char('"'), char('"'), many(char_inside('"'))).map(|StringLiteral(vec)| vec)
}
