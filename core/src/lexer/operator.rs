use util::lexer::Lex;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Operator {
    Equal,
    DoubleEqual,
    NotEqual,
    Colon,
    DoubleColon,
    Dot,
    DoubleDot,
    DotLess,
    GreaterDot,
    GreaterLess,
    Plus,
    DoublePlus,
    Minus,
    Star,
    Slash,
    DoubleSlash,
    Percent,
    Bang,
    Amp,
    Pipe,
    DoubleAmp,
    DoublePipe,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    LeftArrow,
    RightThickArrow,
    Question,
    QuestionDot,
    DoubleQuestion,
}
impl Operator {
    fn new(src: &str) -> Option<Self> {
        Some(match src {
            "=" => Self::Equal,
            "==" => Self::DoubleEqual,
            "!=" => Self::NotEqual,
            ":" => Self::Colon,
            "::" => Self::DoubleColon,
            "." => Self::Dot,
            ".." => Self::DoubleDot,
            ".<" => Self::DotLess,
            ">." => Self::GreaterDot,
            "><" => Self::GreaterLess,
            "+" => Self::Plus,
            "++" => Self::DoublePlus,
            "-" => Self::Minus,
            "*" => Self::Star,
            "/" => Self::Slash,
            "//" => Self::DoubleSlash,
            "%" => Self::Percent,
            "!" => Self::Bang,
            "&" => Self::Amp,
            "|" => Self::Pipe,
            "&&" => Self::DoubleAmp,
            "||" => Self::DoublePipe,
            ">" => Self::Greater,
            "<" => Self::Less,
            ">=" => Self::GreaterEqual,
            "<=" => Self::LessEqual,
            "<-" => Self::LeftArrow,
            "=>" => Self::RightThickArrow,
            "?" => Self::Question,
            "?." => Self::QuestionDot,
            "??" => Self::DoubleQuestion,
            _ => return None,
        })
    }
}
impl<'a> Lex<'a> for Operator {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        let special = src
            .get(..3)
            .map(|val| val == "==>" || val == "<--")
            .unwrap_or(false);
        if !special {
            if let Some(operator) = src.get(..2).and_then(Operator::new) {
                return Some((2, operator));
            }
        }
        let operator = src.get(..1).and_then(Operator::new)?;
        Some((1, operator))
    }
}
