use util::lexer::Lex;
use util::match_lex;

struct Whitespace;
impl<'a> Lex<'a> for Whitespace {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        match src.find(|ch: char| !ch.is_whitespace()) {
            None => Some((src.len(), Self)),
            Some(0) => None,
            Some(i) => Some((i, Self)),
        }
    }
}
struct Comment<'a>(&'a str);
impl<'a> Lex<'a> for Comment<'a> {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        if let Some("--") = src.get(..2) {
            match src[2..].find('\n') {
                None => Some((src.len(), Self(&src[2..]))),
                Some(0) => None,
                Some(i) => Some((i + 3, Self(&src[2..i + 2]))),
            }
        } else {
            None
        }
    }
}
struct Ident<'a>(&'a str);
impl<'a> Lex<'a> for Ident<'a> {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        match src.find(|ch: char| ch != '_' && !ch.is_alphanumeric()) {
            None => Some((src.len(), Self(src))),
            Some(0) => None,
            Some(i) => Some((i, Self(&src[..i]))),
        }
    }
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
struct OpeningBracket(Opening, Bracket);
impl<'a> Lex<'a> for OpeningBracket {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        let (opening, bracket) = match src.get(..1)? {
            "(" => (Opening::Open, Bracket::Paren),
            ")" => (Opening::Close, Bracket::Paren),
            "[" => (Opening::Open, Bracket::Bracket),
            "]" => (Opening::Close, Bracket::Bracket),
            "{" => (Opening::Open, Bracket::Brace),
            "}" => (Opening::Close, Bracket::Brace),
            _ => return None,
        };
        Some((1, Self(opening, bracket)))
    }
}
#[derive(PartialEq, Eq, Debug)]
pub enum Keyword {
    Abort,
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
impl<'a> Lex<'a> for Keyword {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        let (move_state, Ident(ident)) = Lex::lex_first(src)?;
        let keyword = match ident {
            "abort" => Self::Abort,
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
        };
        Some((move_state, keyword))
    }
}
#[derive(PartialEq, Eq, Debug)]
pub enum Separator {
    Comma,
    Semicolon,
}
impl<'a> Lex<'a> for Separator {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        let separator = match src.get(..1)? {
            "," => Self::Comma,
            ";" => Self::Semicolon,
            _ => return None,
        };
        Some((1, separator))
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
impl<'a> Lex<'a> for Operator {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        let special = src
            .get(..3)
            .map(|val| val == "<--" || val == "==>")
            .unwrap_or(false);
        if !special {
            let operator = src.get(..2).and_then(|operator| match operator {
                "==" => Some(Self::DoubleEqual),
                "!=" => Some(Self::NotEqual),
                "::" => Some(Self::DoubleColon),
                ".." => Some(Self::DoubleDot),
                "//" => Some(Self::DoubleSlash),
                "&&" => Some(Self::DoubleAmp),
                "||" => Some(Self::DoublePipe),
                ">>" => Some(Self::DoubleGreater),
                "<<" => Some(Self::DoubleLess),
                ">=" => Some(Self::GreaterEqual),
                "<=" => Some(Self::LessEqual),
                "<-" => Some(Self::LeftArrow),
                "->" => Some(Self::RightArrow),
                "=>" => Some(Self::RightThickArrow),
                "??" => Some(Self::DoubleQuestion),
                _ => None,
            });
            if let Some(operator) = operator {
                return Some((2, operator));
            }
        }
        let operator = src.get(..1)?;
        let operator = match operator {
            "=" => Self::Equal,
            ":" => Self::Colon,
            "." => Self::Dot,
            "+" => Self::Plus,
            "-" => Self::Minus,
            "*" => Self::Star,
            "/" => Self::Slash,
            "%" => Self::Percent,
            "!" => Self::Bang,
            "&" => Self::Amp,
            "|" => Self::Pipe,
            "^" => Self::Caret,
            "~" => Self::Tilde,
            ">" => Self::Greater,
            "<" => Self::Less,
            "?" => Self::Question,
            _ => return None,
        };
        Some((1, operator))
    }
}
struct Num<'a>(&'a str);
impl<'a> Lex<'a> for Num<'a> {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        let mut chars = src.chars();
        let first = chars.next();
        let num = if let Some('.') = first {
            chars.next()
        } else {
            first
        };
        if let Some('0'..='9') = num {
            let mut e = false;
            let mut chars = src[1..].char_indices().peekable();
            while let Some((i, ch)) = chars.next() {
                let resume = match ch {
                    '-' | '+' if e => {
                        e = false;
                        true
                    }
                    'e' | 'E' => {
                        e = true;
                        true
                    }
                    '.' if matches!(chars.peek(), Some((_, '0'..='9'))) => true,
                    '_' => true,
                    ch if ch.is_alphanumeric() => true,
                    _ => false,
                };
                if !resume {
                    return Some((i + 1, Self(&src[..i + 1])));
                }
            }
            Some((src.len(), Self(src)))
        } else {
            None
        }
    }
}
enum Str<'a> {
    Str(&'a str),
    Char(&'a str),
    Unterminated(char, &'a str),
}
impl<'a> Lex<'a> for Str<'a> {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        let mut chars = src.char_indices();
        let (_, first) = chars.next().unwrap();
        if let '\'' | '"' = first {
            let mut escaping = false;
            for (i, ch) in chars {
                if let '\n' = ch {
                    return Some((i + 1, Self::Unterminated(first, &src[..i])));
                } else if escaping {
                    escaping = false;
                    continue;
                } else if let '\\' = ch {
                    escaping = true;
                    continue;
                } else if first == ch {
                    let content = &src[1..i];
                    let token = match first {
                        '\'' => Self::Char(content),
                        '"' => Self::Str(content),
                        _ => unreachable!(),
                    };
                    return Some((i + 1, token));
                }
            }
            Some((src.len(), Self::Unterminated(first, src)))
        } else {
            None
        }
    }
}
#[derive(PartialEq, Eq, Debug)]
pub enum Token<'a> {
    Whitespace,
    Comment(&'a str),
    Num(&'a str),
    Str(&'a str),
    Char(&'a str),
    Keyword(Keyword),
    Identifier(&'a str),
    Separator(Separator),
    Bracket(Opening, Bracket),
    Operator(Operator),
    UnterminatedQuote(char, &'a str),
    InvalidToken(&'a str),
}
impl<'a> Lex<'a> for Token<'a> {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        match_lex! { src;
            Some(Whitespace) => Self::Whitespace,
            Some(Comment(content)) => Self::Comment(content),
            Some(Num(num)) => Self::Num(num),
            Some(keyword) => Self::Keyword(keyword),
            Some(Ident(ident)) => Self::Identifier(ident),
            Some(OpeningBracket(opening, bracket)) => Self::Bracket(opening, bracket),
            Some(separator) => Self::Separator(separator),
            Some(operator) => Self::Operator(operator),
            Some(string) => match string {
                Str::Str(content) => Self::Str(content),
                Str::Char(content) => Self::Char(content),
                Str::Unterminated(ch, src) => Self::UnterminatedQuote(ch, src),
            },
            Some(Num(num)) => Self::Num(num),
            else src => {
                let len = src.chars().next().unwrap().len_utf8();
                Some((len, Self::InvalidToken(&src[..len])))
            }
        }
    }
}
#[cfg(test)]
mod test {
    use super::Bracket;
    use super::Keyword;
    use super::Opening;
    use super::Operator;
    use super::Separator;
    use super::Token;
    use util::lexer::Lex;
    #[test]
    fn simple_lex() {
        let vec: Vec<_> =
            Token::lex("-- comment\n identifier true_false null => + ( ) ; <--").collect();
        assert_eq!(
            vec,
            vec![
                Token::Comment(" comment"),
                Token::Whitespace,
                Token::Identifier("identifier"),
                Token::Whitespace,
                Token::Identifier("true_false"),
                Token::Whitespace,
                Token::Keyword(Keyword::Null),
                Token::Whitespace,
                Token::Operator(Operator::RightThickArrow),
                Token::Whitespace,
                Token::Operator(Operator::Plus),
                Token::Whitespace,
                Token::Bracket(Opening::Open, Bracket::Paren),
                Token::Whitespace,
                Token::Bracket(Opening::Close, Bracket::Paren),
                Token::Whitespace,
                Token::Separator(Separator::Semicolon),
                Token::Whitespace,
                Token::Operator(Operator::Less),
                Token::Comment(""),
            ],
        );
    }
    #[test]
    fn lex_string() {
        let vec: Vec<_> = Token::lex(
            r#"
"hello world"
"hello \"world\""
"hello world \\"
"#,
        )
        .collect();
        assert_eq!(
            vec,
            vec![
                Token::Whitespace,
                Token::Str("hello world"),
                Token::Whitespace,
                Token::Str(r#"hello \"world\""#),
                Token::Whitespace,
                Token::Str(r"hello world \\"),
                Token::Whitespace,
            ],
        );
    }
    #[test]
    fn lex_number() {
        let vec: Vec<_> = Token::lex(
            r#"
12
5.
.5
1e+10
1e-10
"#,
        )
        .collect();
        assert_eq!(
            vec,
            vec![
                Token::Whitespace,
                Token::Num("12"),
                Token::Whitespace,
                Token::Num("5"),
                Token::Operator(Operator::Dot),
                Token::Whitespace,
                Token::Num(".5"),
                Token::Whitespace,
                Token::Num("1e+10"),
                Token::Whitespace,
                Token::Num("1e-10"),
                Token::Whitespace,
            ],
        );
    }
}
