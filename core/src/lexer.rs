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

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
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
