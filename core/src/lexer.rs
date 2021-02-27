use crate::lexer::bracket::OpeningBracket;
use crate::lexer::comment::Comment;
use crate::lexer::ident::Ident;
use crate::lexer::integer::Int;
use crate::lexer::number::Num;
use crate::lexer::string::Str;
use crate::lexer::underscore::Underscore;
use crate::lexer::whitespace::Whitespace;
use std::num::NonZeroUsize;
use util::lexer::Lex;
use util::lexer::LexFilter;
use util::match_lex;

pub use crate::lexer::bracket::Bracket;
pub use crate::lexer::bracket::Opening;
pub use crate::lexer::float::Float;
pub use crate::lexer::float::Sign;
pub use crate::lexer::integer::Radix;
pub use crate::lexer::keyword::Keyword;
pub use crate::lexer::operator::Operator;
pub use crate::lexer::separator::Separator;

mod bracket;
mod comment;
mod float;
mod ident;
mod integer;
mod keyword;
mod number;
mod operator;
mod separator;
mod string;
mod underscore;
mod whitespace;

const INSIGNIFICANT_DIGIT_START: &[char] = &['_', '0'];

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Token<'a> {
    Whitespace,
    Comment,
    Int(Radix, &'a str),
    Float(Float<'a>),
    Str(&'a str),
    Char(&'a str),
    Keyword(Keyword),
    Underscore,
    Ident,
    Separator(Separator),
    Bracket(Opening, Bracket),
    Operator(Operator),
    InvalidNumber,
    UnterminatedQuote,
    Unknown,
}
impl<'a> Lex<'a> for Token<'a> {
    fn lex_first(src: &'a str) -> Option<(NonZeroUsize, Self)> {
        match_lex! { src;
            Whitespace => Self::Whitespace,
            keyword => Self::Keyword(keyword),
            Underscore => Self::Underscore,
            Ident => Self::Ident,
            float => Self::Float(float),
            Int(radix, int) => Self::Int(radix, int),
            Num => Self::InvalidNumber,
            Comment => Self::Comment,
            operator => Self::Operator(operator),
            OpeningBracket(opening, bracket) => Self::Bracket(opening, bracket),
            separator => Self::Separator(separator),
            Str::Str(content) => Self::Str(content),
            Str::Char(content) => Self::Char(content),
            Str::Unterminated => Self::UnterminatedQuote,
            => else src => {
                let len = src.chars().next().unwrap().len_utf8();
                Some((NonZeroUsize::new(len).unwrap(), Self::Unknown))
            }
        }
    }
}
impl<'a> LexFilter<'a> for Token<'a> {
    fn significant(&self) -> bool {
        !matches!(self, Self::Whitespace | Self::Comment)
    }
}
#[cfg(test)]
mod test {
    use super::Bracket;
    use super::Float;
    use super::Keyword;
    use super::Opening;
    use super::Operator;
    use super::Radix;
    use super::Separator;
    use super::Sign;
    use super::Token;
    use util::assert_iter;
    use util::lexer::LexFilter;
    #[test]
    fn simple_lex() {
        assert_iter! {
            Token::lex_span("-- comment\n identifier_123 true_false _ null => + ( ) ;"),
            ("identifier_123", Token::Ident),
            ("true_false", Token::Ident),
            ("_", Token::Underscore),
            ("null", Token::Keyword(Keyword::Null)),
            ("=>", Token::Operator(Operator::RightThickArrow)),
            ("+", Token::Operator(Operator::Plus)),
            ("(", Token::Bracket(Opening::Open, Bracket::Parenthesis)),
            (")", Token::Bracket(Opening::Close, Bracket::Parenthesis)),
            (";", Token::Separator(Separator::Semicolon)),
        }
    }
    #[test]
    fn lex_string() {
        assert_iter! {
            Token::lex_span(r#""hello world" "hello \"world\"" "hello world \\""#),
            (r#""hello world""#, Token::Str("hello world")),
            (r#""hello \"world\"""#, Token::Str(r#"hello \"world\""#)),
            (r#""hello world \\""#, Token::Str(r"hello world \\")),
        }
    }
    #[test]
    fn lex_integer() {
        assert_iter! {
            Token::lex_span("12 1_000_000 0x_18e 0o_127 0b_11110000"),
            ("12", Token::Int(Radix::Dec, "12")),
            ("1_000_000", Token::Int(Radix::Dec, "1_000_000")),
            ("0x_18e", Token::Int(Radix::Hex, "18e")),
            ("0o_127", Token::Int(Radix::Oct, "127")),
            ("0b_11110000", Token::Int(Radix::Bin, "11110000")),
        }
    }
    #[test]
    fn lex_float() {
        assert_iter! {
            Token::lex_span("1.2 3. .4 5.6_e7 8_e+9 10_e-11"),
            ("1.2", Token::Float(Float {
                whole: "1",
                decimal: "2",
                exp_sign: Sign::Plus,
                exp: "",
            })),
            ("3", Token::Int(Radix::Dec,"3")),
            (".", Token::Operator(Operator::Dot)),
            (".4", Token::Float(Float {
                whole: "",
                decimal: "4",
                exp_sign: Sign::Plus,
                exp: "",
            })),
            ("5.6_e7", Token::Float(Float {
                whole: "5",
                decimal: "6",
                exp_sign: Sign::Plus,
                exp: "7",
            })),
            ("8_e+9", Token::Float(Float {
                whole: "8_",
                decimal: "",
                exp_sign: Sign::Plus,
                exp: "9",
            })),
            ("10_e-11", Token::Float(Float {
                whole: "10_",
                decimal: "",
                exp_sign: Sign::Minus,
                exp: "11",
            })),
        }
    }
}
