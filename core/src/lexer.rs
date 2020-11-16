pub use bracket::Bracket;
pub use bracket::Opening;
use bracket::OpeningBracket;
use comment::Comment;
use ident::Ident;
pub use keyword::Keyword;
use number::Num;
pub use operator::Operator;
pub use separator::Separator;
use string::Str;
use util::lexer::Lex;
use util::lexer::LexFilter;
use util::match_lex;
use whitespace::Whitespace;

mod bracket;
mod comment;
mod ident;
mod keyword;
mod number;
mod operator;
mod separator;
mod string;
mod whitespace;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Token<'a> {
    Whitespace,
    Comment,
    Num,
    Str(&'a str),
    Char(&'a str),
    Keyword(Keyword),
    Ident,
    Separator(Separator),
    Bracket(Opening, Bracket),
    Operator(Operator),
    UnterminatedQuote,
    Unknown,
}
impl<'a> Lex<'a> for Token<'a> {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        match_lex! { src;
            Whitespace => Self::Whitespace,
            Num => Self::Num,
            keyword => Self::Keyword(keyword),
            Ident => Self::Ident,
            Comment => Self::Comment,
            operator => Self::Operator(operator),
            OpeningBracket(opening, bracket) => Self::Bracket(opening, bracket),
            separator => Self::Separator(separator),
            Str::Str(content) => Self::Str(content),
            Str::Char(content) => Self::Char(content),
            Str::Unterminated => Self::UnterminatedQuote,
            => else src => {
                let len = src.chars().next().unwrap().len_utf8();
                Some((len, Self::Unknown))
            }
        }
    }
}
impl<'a> LexFilter<'a> for Token<'a> {
    fn significant(&self) -> bool {
        match self {
            Self::Whitespace => false,
            Self::Comment => false,
            _ => true,
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
    use util::assert_iter;
    use util::lexer::LexFilter;
    #[test]
    fn simple_lex() {
        assert_iter! {
            Token::lex_span("-- comment\n identifier_123 true_false null => + ( ) ;"),
            ("identifier_123", Token::Ident),
            ("true_false", Token::Ident),
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
    fn lex_number() {
        assert_iter! {
            Token::lex_span("12 5. .5 1e+10 1e-10 0xe+10"),
            ("12", Token::Num),
            ("5", Token::Num),
            (".", Token::Operator(Operator::Dot)),
            (".5", Token::Num),
            ("1e+10", Token::Num),
            ("1e-10", Token::Num),
            ("0xe", Token::Num),
            ("+", Token::Operator(Operator::Plus)),
            ("10", Token::Num),
        }
    }
}
