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
            Some(Comment) => Self::Comment,
            Some(Num) => Self::Num,
            Some(keyword) => Self::Keyword(keyword),
            Some(Ident) => Self::Identifier,
            Some(OpeningBracket(opening, bracket)) => Self::Bracket(opening, bracket),
            Some(separator) => Self::Separator(separator),
            Some(operator) => Self::Operator(operator),
            Some(string) => match string {
                Str::Str(content) => Self::Str(content),
                Str::Char(content) => Self::Char(content),
                Str::Unterminated => Self::UnterminatedQuote,
            },
            Some(Num) => Self::Num,
            else src => {
                let len = src.chars().next().unwrap().len_utf8();
                Some((len, Self::Invalid))
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
            Token::lex_span("-- comment\n identifier_123 true_false null => + ( ) ; <--").collect();
        assert_eq!(
            vec,
            vec![
                ("-- comment\n", Token::Comment),
                (" ", Token::Whitespace),
                ("identifier_123", Token::Identifier),
                (" ", Token::Whitespace),
                ("true_false", Token::Identifier),
                (" ", Token::Whitespace),
                ("null", Token::Keyword(Keyword::Null)),
                (" ", Token::Whitespace),
                ("=>", Token::Operator(Operator::RightThickArrow)),
                (" ", Token::Whitespace),
                ("+", Token::Operator(Operator::Plus)),
                (" ", Token::Whitespace),
                ("(", Token::Bracket(Opening::Open, Bracket::Paren)),
                (" ", Token::Whitespace),
                (")", Token::Bracket(Opening::Close, Bracket::Paren)),
                (" ", Token::Whitespace),
                (";", Token::Separator(Separator::Semicolon)),
                (" ", Token::Whitespace),
                ("<", Token::Operator(Operator::Less)),
                ("--", Token::Comment),
            ],
        );
    }
    #[test]
    fn lex_string() {
        let vec: Vec<_> = Token::lex_span(
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
                ("\n", Token::Whitespace),
                (r#""hello world""#, Token::Str("hello world")),
                ("\n", Token::Whitespace),
                (r#""hello \"world\"""#, Token::Str(r#"hello \"world\""#)),
                ("\n", Token::Whitespace),
                (r#""hello world \\""#, Token::Str(r"hello world \\")),
                ("\n", Token::Whitespace),
            ],
        );
    }
    #[test]
    fn lex_number() {
        let vec: Vec<_> = Token::lex_span(
            r"
12
5.
.5
1e+10
1e-10
",
        )
        .collect();
        assert_eq!(
            vec,
            vec![
                ("\n", Token::Whitespace),
                ("12", Token::Num),
                ("\n", Token::Whitespace),
                ("5", Token::Num),
                (".", Token::Operator(Operator::Dot)),
                ("\n", Token::Whitespace),
                (".5", Token::Num),
                ("\n", Token::Whitespace),
                ("1e+10", Token::Num),
                ("\n", Token::Whitespace),
                ("1e-10", Token::Num),
                ("\n", Token::Whitespace),
            ],
        );
    }
}
