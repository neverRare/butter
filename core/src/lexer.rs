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
    True,
    False,
    Null,
    Void,
    Clone,
    If,
    Else,
    Match,
    For,
    Loop,
    While,
}
impl Keyword {
    fn from_str(bracket: &str) -> Option<Self> {
        Some(match bracket {
            "true" => Self::True,
            "false" => Self::False,
            "null" => Self::Null,
            "void" => Self::Void,
            "clone" => Self::Clone,
            "if" => Self::If,
            "else" => Self::Else,
            "match" => Self::Match,
            "for" => Self::For,
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
    Asterisk,
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
            "*" => Self::Asterisk,
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
pub struct Tokens<'a> {
    src: &'a str,
    i: usize,
}
impl<'a> Tokens<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { src, i: 0 }
    }
}
impl<'a> Iterator for Tokens<'a> {
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Self::Item> {
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
            }
            if first.is_alphabetic() {
                let mut i = first.len_utf8();
                for ch in chars {
                    if ch.is_alphanumeric() {
                        i += ch.len_utf8();
                    }
                }
                let ident = &src[..i];
                self.i = i;
                break Some(match Keyword::from_str(ident) {
                    Some(keyword) => Token::Keyword(keyword),
                    None => Token::Identifier(ident),
                });
            }
            if let Some("--") = src.get(0..2) {
                let rest = &src[2..];
                match rest.find('\n') {
                    Some(index) => {
                        i += 3 + index;
                        continue;
                    }
                    None => break None,
                }
            }
        }
    }
}