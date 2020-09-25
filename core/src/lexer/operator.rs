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
    QuestionDot,
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
                ".<" => Some(Self::DotLess),
                ">." => Some(Self::GreaterDot),
                "><" => Some(Self::GreaterLess),
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
                "?." => Some(Self::QuestionDot),
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
