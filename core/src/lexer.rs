use bracket::Bracket;
use bracket::Opening;
use bracket::OpeningBracket;
use comment::Comment;
use ident::Ident;
use keyword::Keyword;
use number::Num;
use operator::Operator;
use separator::Separator;
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
    Identifier,
    Separator(Separator),
    Bracket(Opening, Bracket),
    Operator(Operator),
    UnterminatedQuote,
    Invalid,
}
impl<'a> Lex<'a> for Token<'a> {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        match_lex! { src;
            Some(Whitespace) => Self::Whitespace,
            Some(Num) => Self::Num,
            Some(keyword) => Self::Keyword(keyword),
            Some(Ident) => Self::Identifier,
            Some(Comment) => Self::Comment,
            Some(operator) => Self::Operator(operator),
            Some(OpeningBracket(opening, bracket)) => Self::Bracket(opening, bracket),
            Some(separator) => Self::Separator(separator),
            Some(Str::Str(content)) => Self::Str(content),
            Some(Str::Char(content)) => Self::Char(content),
            Some(Str::Unterminated) => Self::UnterminatedQuote,
            else src => {
                let len = src.chars().next().unwrap().len_utf8();
                Some((len, Self::Invalid))
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
            Token::lex_span("-- comment\n identifier_123 true_false null => + ( ) ; <--"),
            ("identifier_123", Token::Identifier),
            ("true_false", Token::Identifier),
            ("null", Token::Keyword(Keyword::Null)),
            ("=>", Token::Operator(Operator::RightThickArrow)),
            ("+", Token::Operator(Operator::Plus)),
            ("(", Token::Bracket(Opening::Open, Bracket::Paren)),
            (")", Token::Bracket(Opening::Close, Bracket::Paren)),
            (";", Token::Separator(Separator::Semicolon)),
            ("<", Token::Operator(Operator::Less)),
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
            Token::lex_span("12 5. .5 1e+10 1e-10"),
            ("12", Token::Num),
            ("5", Token::Num),
            (".", Token::Operator(Operator::Dot)),
            (".5", Token::Num),
            ("1e+10", Token::Num),
            ("1e-10", Token::Num),
        }
    }
}
