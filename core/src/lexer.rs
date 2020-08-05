use crate::lexer::number::InvalidChar;
use crate::lexer::number::NumError;
use crate::lexer::number::OverflowError;
use crate::lexer::string::EscapeError;
use crate::lexer::string::StrError;
use crate::span::Span;
use number::parse_number;
use string::parse_string;

mod number;
mod string;

#[derive(PartialEq, Debug)]
pub enum Num {
    UInt(u64),
    Int(i64),
    Float(f64),
}
#[derive(PartialEq, Eq, Debug)]
pub enum Bracket {
    Paren,
    Bracket,
    Brace,
}
#[derive(PartialEq, Eq, Debug)]
pub enum Opening {
    Open,
    Close,
}
fn get_bracket(bracket: &str) -> Option<(Opening, Bracket)> {
    Some(match bracket {
        "(" => (Opening::Open, Bracket::Paren),
        ")" => (Opening::Close, Bracket::Paren),
        "[" => (Opening::Open, Bracket::Bracket),
        "]" => (Opening::Close, Bracket::Bracket),
        "{" => (Opening::Open, Bracket::Brace),
        "}" => (Opening::Close, Bracket::Brace),
        _ => return None,
    })
}
#[derive(PartialEq, Eq, Debug)]
pub enum Keyword {
    Move,
    True,
    False,
    Null,
    Clone,
    If,
    Else,
    For,
    In,
    Loop,
    While,
    Break,
    Continue,
}
impl Keyword {
    fn from_str(bracket: &str) -> Option<Self> {
        Some(match bracket {
            "move" => Self::Move,
            "true" => Self::True,
            "false" => Self::False,
            "null" => Self::Null,
            "clone" => Self::Clone,
            "if" => Self::If,
            "else" => Self::Else,
            "for" => Self::For,
            "in" => Self::In,
            "loop" => Self::Loop,
            "while" => Self::While,
            "break" => Self::Break,
            "continue" => Self::Continue,
            _ => return None,
        })
    }
}
#[derive(PartialEq, Eq, Debug)]
pub enum Separator {
    Comma,
    Semicolon,
}
impl Separator {
    fn from_str(bracket: &str) -> Option<Self> {
        Some(match bracket {
            "," => Self::Comma,
            ";" => Self::Semicolon,
            _ => return None,
        })
    }
}
#[derive(PartialEq, Eq, Debug)]
pub enum Operator {
    Equal,
    DoubleEqual,
    NotEqual,
    Colon,
    DoubleColon,
    Dot,
    DoubleDot,
    Plus,
    Minus,
    Star,
    Slash,
    DoubleSlash,
    Percent,
    Bang,
    Amp,
    Pipe,
    Caret,
    Tilde,
    DoubleAmp,
    DoublePipe,
    Greater,
    Less,
    DoubleGreater,
    DoubleLess,
    GreaterEqual,
    LessEqual,
    LeftArrow,
    RightArrow,
    RightThickArrow,
    Question,
    DoubleQuestion,
}
impl Operator {
    fn from_str(operator: &str) -> Option<Self> {
        Some(match operator {
            "=" => Self::Equal,
            "==" => Self::DoubleEqual,
            "!=" => Self::NotEqual,
            ":" => Self::Colon,
            "::" => Self::DoubleColon,
            "." => Self::Dot,
            ".." => Self::DoubleDot,
            "+" => Self::Plus,
            "-" => Self::Minus,
            "*" => Self::Star,
            "/" => Self::Slash,
            "//" => Self::DoubleSlash,
            "%" => Self::Percent,
            "!" => Self::Bang,
            "&" => Self::Amp,
            "|" => Self::Pipe,
            "^" => Self::Caret,
            "~" => Self::Tilde,
            "&&" => Self::DoubleAmp,
            "||" => Self::DoublePipe,
            ">" => Self::Greater,
            "<" => Self::Less,
            ">>" => Self::DoubleGreater,
            "<<" => Self::DoubleLess,
            ">=" => Self::GreaterEqual,
            "<=" => Self::LessEqual,
            "<-" => Self::LeftArrow,
            "->" => Self::RightArrow,
            "=>" => Self::RightThickArrow,
            "?" => Self::Question,
            "??" => Self::DoubleQuestion,
            _ => return None,
        })
    }
}
#[derive(PartialEq, Debug)]
pub enum Token<'a> {
    Num(Num),
    Str(Vec<u8>),
    Char(u8),
    Keyword(Keyword),
    Identifier(&'a str),
    Separator(Separator),
    Bracket(Opening, Bracket),
    Operator(Operator),
}
impl<'a> Token<'a> {
    pub fn lex(src: &'a str) -> Result<Vec<Self>, Vec<(Span, LexerError)>> {
        let mut res: Result<_, Vec<(Span, LexerError)>> = Ok(vec![]);
        for (span, token) in TokenSpans::new(src) {
            match token {
                Ok(token) => {
                    if let Ok(mut vec) = res {
                        vec.push(token);
                        res = Ok(vec);
                    }
                }
                Err(err) => {
                    let err = (span, err);
                    if let Err(mut vec) = res {
                        vec.push(err);
                        res = Err(vec);
                    } else {
                        res = Err(vec![err]);
                    }
                }
            }
        }
        res
    }
}
#[derive(PartialEq, Eq, Debug)]
pub enum LexerError<'a> {
    UnknownChar(Span<'a>),
    UnterminatedQuote(Span<'a>, char),
    InvalidEscape(Vec<(Span<'a>, EscapeError)>),
    CharNotOne(Span<'a>),
    InvalidChar(Vec<(Span<'a>, InvalidChar)>),
    Overflow(Span<'a>, OverflowError),
}
pub struct TokenSpans<'a> {
    src: &'a str,
    i: usize,
    done: bool,
}
impl<'a> TokenSpans<'a> {
    pub fn new(src: &'a str) -> Self {
        TokenSpans {
            src,
            i: 0,
            done: false,
        }
    }
}
impl<'a> Iterator for TokenSpans<'a> {
    type Item = (Span<'a>, Result<Token<'a>, LexerError<'a>>);
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            let mut i = self.i;
            let (len, result) = loop {
                let src = &self.src[i..];
                let first = match src.chars().next() {
                    Some(val) => val,
                    None => {
                        self.done = true;
                        return None;
                    }
                };
                let first_len = first.len_utf8();
                let rest = &src[first_len..];
                if first.is_whitespace() {
                    match rest.find(|ch: char| !ch.is_whitespace()) {
                        Some(ind) => {
                            i += first_len + ind;
                            continue;
                        }
                        None => return None,
                    }
                } else if first.is_alphabetic() {
                    let len = first_len
                        + rest
                            .find(|ch: char| !ch.is_alphanumeric())
                            .unwrap_or_default();
                    let ident = &src[..len];
                    break (
                        len,
                        Ok(match Keyword::from_str(ident) {
                            Some(keyword) => Token::Keyword(keyword),
                            None => Token::Identifier(ident),
                        }),
                    );
                } else if let ('0'..='9', _) | ('.', Some('0'..='9')) = (first, rest.chars().next())
                {
                    let (len, num) = parse_number(src);
                    break (
                        len,
                        match num {
                            Ok(num) => Ok(Token::Num(num)),
                            Err(err) => Err(match err {
                                NumError::InvalidChar(spans) => LexerError::InvalidChar(
                                    spans
                                        .into_iter()
                                        .map(|(span, err)| (span.fit_from(self.src), err))
                                        .collect(),
                                ),
                                NumError::Overflow(err) => {
                                    LexerError::Overflow(Span::new(self.src, i..i + len), err)
                                }
                            }),
                        },
                    );
                } else if let '\'' | '"' = first {
                    let rest = match rest.find('\n') {
                        Some(ind) => &rest[..ind],
                        None => rest,
                    };
                    let (len, token) = parse_string(first, rest);
                    break (
                        len + 2,
                        match token {
                            Ok(val) => match first {
                                '\'' if val.len() == 1 => Ok(Token::Char(val[0])),
                                '\'' => {
                                    Err(LexerError::CharNotOne(Span::new(self.src, i..i + len + 2)))
                                }
                                '"' => Ok(Token::Str(val)),
                                _ => unreachable!(),
                            },
                            Err(err) => match err {
                                StrError::InvalidEscape(vec) => Err(LexerError::InvalidEscape(
                                    vec.into_iter()
                                        .map(|(span, err)| (span.fit_from(self.src), err))
                                        .collect(),
                                )),
                                StrError::Unterminated => {
                                    self.done = true;
                                    Err(LexerError::UnterminatedQuote(
                                        Span::new(self.src, i..i + len + 2),
                                        first,
                                    ))
                                }
                            },
                        },
                    );
                } else if let Some("<--") = src.get(0..3) {
                    break (1, Ok(Token::Operator(Operator::Less)));
                } else if let Some(val) = src.get(0..2) {
                    if val == "--" {
                        let rest = &src[2..];
                        match rest.find('\n') {
                            Some(index) => {
                                i += 3 + index;
                                continue;
                            }
                            None => {
                                self.done = true;
                                return None;
                            }
                        }
                    } else if let Some(val) = Operator::from_str(val) {
                        break (2, Ok(Token::Operator(val)));
                    }
                }
                if let Some(val) = src.get(0..1) {
                    let token = if let Some(val) = Operator::from_str(val) {
                        Some(Token::Operator(val))
                    } else if let Some(val) = Separator::from_str(val) {
                        Some(Token::Separator(val))
                    } else if let Some((opening, bracket)) = get_bracket(val) {
                        Some(Token::Bracket(opening, bracket))
                    } else {
                        None
                    };
                    if let Some(token) = token {
                        break (i, Ok(token));
                    }
                }
                break (
                    first_len,
                    Err(LexerError::UnknownChar(Span::new(
                        self.src,
                        i..i + first_len,
                    ))),
                );
            };
            self.i = i + len;
            Some((Span::new(self.src, i..i + len), result))
        }
    }
}
#[cfg(test)]
mod test {
    use super::{Bracket, Keyword, Num, Opening, Operator, Separator, Token};
    #[test]
    fn simple_lex() {
        assert_eq!(
            Token::lex("-- comment\n identifier truefalse null => + ( ) ; <--"),
            Ok(vec![
                Token::Identifier("identifier"),
                Token::Identifier("truefalse"),
                Token::Keyword(Keyword::Null),
                Token::Operator(Operator::RightThickArrow),
                Token::Operator(Operator::Plus),
                Token::Bracket(Opening::Open, Bracket::Paren),
                Token::Bracket(Opening::Close, Bracket::Paren),
                Token::Separator(Separator::Semicolon),
                Token::Operator(Operator::Less),
            ]),
        );
    }
    #[test]
    fn lex_string() {
        assert_eq!(
            Token::lex(
                r#"
"hello world"
"hello \"world\""
"hello world \\"
'a'
'\''
'\\'
'\x7A'
""""
'a''a'
"#
            ),
            Ok(vec![
                Token::Str(b"hello world".to_vec()),
                Token::Str(b"hello \"world\"".to_vec()),
                Token::Str(b"hello world \\".to_vec()),
                Token::Char(b'a'),
                Token::Char(b'\''),
                Token::Char(b'\\'),
                Token::Char(b'\x7A'),
                Token::Str(vec![]),
                Token::Str(vec![]),
                Token::Char(b'a'),
                Token::Char(b'a'),
            ]),
        );
    }
    #[test]
    fn lex_number() {
        assert_eq!(
            Token::lex(
                r#"
12
0.5
0xff
0b11110000
0o127
1_000_000
4e-7
4e7
4e70
2.
.5
"#
            ),
            Ok(vec![
                Num::UInt(12),
                Num::Float(0.5),
                Num::UInt(0xff),
                Num::UInt(0b11110000),
                Num::UInt(0o127),
                Num::UInt(1_000_000),
                Num::Float(4e-7),
                Num::UInt(40_000_000),
                Num::Float(4e70),
                Num::UInt(2),
            ]
            .into_iter()
            .map(Token::Num)
            .chain(vec![
                Token::Operator(Operator::Dot),
                Token::Num(Num::Float(0.5)),
            ])
            .collect()),
        );
    }
}
