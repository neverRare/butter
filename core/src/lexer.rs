use crate::error::ErrorSpan;

pub enum Num {
    UInt(u64),
    Int(i64),
    Float(f64),
}
pub enum Bracket {
    Paren,
    Bracket,
    Brace,
}
pub enum Opening {
    Open,
    Close,
}
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
            _ => return None,
        })
    }
}
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
fn get_number_str(src: &str) -> Num {
    todo!()
}
fn parse_string(src: &str) -> Vec<u8> {
    todo!()
}
pub enum LexerError {
    UnknownChar,
}
pub struct Tokens<'a> {
    src: &'a str,
    i: usize,
    erred: bool,
}
impl<'a> Tokens<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            i: 0,
            erred: false,
        }
    }
}
impl<'a> Iterator for Tokens<'a> {
    type Item = Result<Token<'a>, ErrorSpan<'a, LexerError>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.erred {
            None
        } else {
            let mut i = self.i;
            'outer: loop {
                let src = &self.src[i..];
                let mut chars = src.chars();
                let first = chars.next();
                let first = match first {
                    Some(val) => val,
                    None => break None,
                };
                if first.is_whitespace() {
                    i += first.len_utf8();
                    for ch in chars {
                        if ch.is_whitespace() {
                            i += ch.len_utf8();
                        } else {
                            continue 'outer;
                        }
                    }
                    break None;
                } else if first.is_alphabetic() {
                    let mut len = first.len_utf8();
                    for ch in chars {
                        if ch.is_alphanumeric() {
                            len += ch.len_utf8();
                        } else {
                            break;
                        }
                    }
                    let ident = &src[..len];
                    self.i = i + len;
                    break Some(Ok(match Keyword::from_str(ident) {
                        Some(keyword) => Token::Keyword(keyword),
                        None => Token::Identifier(ident),
                    }));
                } else if let Some(val) = src.get(0..2) {
                    if val == "--" {
                        let rest = &src[2..];
                        match rest.find('\n') {
                            Some(index) => {
                                i += 3 + index;
                                continue;
                            }
                            None => break None,
                        }
                    } else if let Some(val) = Operator::from_str(val) {
                        self.i = i + 2;
                        break Some(Ok(Token::Operator(val)));
                    }
                }
                if let Some(val) = src.get(0..1) {
                    if let Some(val) = Operator::from_str(val) {
                        self.i = i + 1;
                        break Some(Ok(Token::Operator(val)));
                    } else if let Some(val) = Separator::from_str(val) {
                        self.i = i + 1;
                        break Some(Ok(Token::Separator(val)));
                    }
                }
                self.erred = true;
                break Some(Err(ErrorSpan::new(
                    LexerError::UnknownChar,
                    self.src,
                    i,
                    i + first.len_utf8(),
                )));
            }
        }
    }
}
