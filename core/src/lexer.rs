use crate::number::parse_number;
use crate::span::Span;
use crate::string::parse_string;

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
#[derive(PartialEq, Eq, Debug)]
pub enum LexerError {
    UnknownChar,
    UnterminatedQuote,
    CharNotOne,
    InvalidEscape,
    InvalidCharOnHex,
    InvalidCharOnOct,
    InvalidCharOnBin,
    DecimalOnInt,
    DecimalOnMagnitude,
    InvalidCharOnNum,
    DoubleMagnitude,
    DoubleDecimal,
    MagnitudeOverflow,
    IntegerOverflow,
}
pub struct Tokens<'a> {
    src: &'a str,
    i: usize,
    done: bool,
}
impl<'a> Tokens<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            i: 0,
            done: false,
        }
    }
}
impl<'a> Iterator for Tokens<'a> {
    type Item = Result<Token<'a>, Span<'a, LexerError>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            let mut i = self.i;
            'outer: loop {
                let src = &self.src[i..];
                let first = match src.chars().next() {
                    Some(val) => val,
                    None => {
                        self.done = true;
                        break None;
                    }
                };
                let first_len = first.len_utf8();
                let rest = &src[first_len..];
                if first.is_whitespace() {
                    for (ind, ch) in rest.char_indices() {
                        if !ch.is_whitespace() {
                            i += first_len + ind;
                            continue 'outer;
                        }
                    }
                    break None;
                } else if first.is_alphabetic() {
                    let mut len = first_len;
                    for (ind, ch) in rest.char_indices() {
                        if !ch.is_alphanumeric() {
                            len = first_len + ind;
                            break;
                        }
                    }
                    let ident = &src[..len];
                    self.i = i + len;
                    break Some(Ok(match Keyword::from_str(ident) {
                        Some(keyword) => Token::Keyword(keyword),
                        None => Token::Identifier(ident),
                    }));
                } else if matches!(first, '0'..='9')
                    || (first == '.' && matches!(rest.chars().next(), Some('0'..='9')))
                {
                    match parse_number(src) {
                        Ok((len, num)) => {
                            self.i = i + len;
                            return Some(Ok(Token::Num(num)));
                        }
                        Err(span) => {
                            self.done = true;
                            return Some(Err(span.fit_from(src)));
                        }
                    }
                } else if let '\'' | '"' = first {
                    let rest = match rest.find('\n') {
                        Some(ind) => &rest[..ind],
                        None => rest,
                    };
                    break Some(match parse_string(first, rest) {
                        Ok((len, val)) => {
                            self.i = i + len + 2;
                            match first {
                                '\'' if val.len() == 1 => Ok(Token::Char(val[0])),
                                '\'' => {
                                    self.done = true;
                                    Err(Span::new(self.src, LexerError::CharNotOne, i, i + len + 2))
                                }
                                '"' => Ok(Token::Str(val)),
                                _ => unreachable!(),
                            }
                        }
                        Err(span) => {
                            self.done = true;
                            Err(span.fit_from(src))
                        }
                    });
                } else if let Some("<--") = src.get(0..3) {
                    self.i = i + 1;
                    break Some(Ok(Token::Operator(Operator::Less)));
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
                                break None;
                            }
                        }
                    } else if let Some(val) = Operator::from_str(val) {
                        self.i = i + 2;
                        break Some(Ok(Token::Operator(val)));
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
                        self.i = i + 1;
                        break Some(Ok(token));
                    }
                }
                self.done = true;
                break Some(Err(Span::new(
                    self.src,
                    LexerError::UnknownChar,
                    i,
                    i + first.len_utf8(),
                )));
            }
        }
    }
}
#[cfg(test)]
mod test {
    use super::{Token, Tokens};
    macro_rules! assert_code {
        (@ $tokens:expr, ignore) => {};
        (@ $tokens:expr, $type:ident) => {
            let next = $tokens.next();
            assert!(
                matches!(next, Some(Ok(Token::$type{..}))),
                "{:?} expected to be {}",
                next,
                stringify!($type),
            );
        };
        (@ $tokens:expr, == $more:literal) => {
            for token in Tokens::new($more) {
                assert_eq!($tokens.next(), Some(token));
            }
        };
        ($($($type:ident:)? $token:literal $(== $more:literal)?;)*) => {
            let mut tokens = Tokens::new(concat!($($token, " "),*));
            $(
                assert_code!(@ tokens, $($type)? $(== $more)?);
            )*
            assert_eq!(None, tokens.next());
        };
    }
    #[test]
    fn simple_lex() {
        assert_code! {
            ignore: "-- comment\n";
            Identifier: "identifier";
            Identifier: "truefalse";
            Keyword: "null";
            Operator: "=>";
            Operator: "+";
            Bracket: "(";
            Bracket: ")";
            Separator: ";";
            "<--" == "<";
        }
    }
    #[test]
    fn lex_string() {
        assert_code! {
            Str: r#""hello world""#;
            Str: r#""hello \"world\"""#;
            Str: r#""hello world \\""#;
            Char: "'a'";
            Char: r"'\''";
            Char: r"'\\'";
            r#"'\x7A'"# == r#"'z'"#;
            r#""""""# == r#""" """#;
            "'a''a'" == "'a' 'a'";
        }
    }
    #[test]
    fn lex_number() {
        assert_code! {
            Num: "12";
            Num: "0.5";
            Num: "0xff";
            Num: "0b11110000";
            Num: "0o127";
            Num: "1_000_000";
            Num: "4e-7";
            "2." == "2 .";
            ".5" == "0.5";
            "0xff" == "255";
            "0b11110000" == "240";
            "0o127" == "87";
            "1_000_000" == "1000000";
            "4e-7" == ".0000004";
        }
    }
}
