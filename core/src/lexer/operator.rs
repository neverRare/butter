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
    SlashBang,
    SlashQuestion,
    DoubleSlash,
    DoubleSlashBang,
    DoubleSlashQuestion,
    Percent,
    PercentBang,
    PercentQuestion,
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
impl<'a> Lex<'a> for Operator {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        let src3 = src.get(..3);
        if let Some(operator) = src3 {
            let operator = match operator {
                "//!" => Some(Self::DoubleSlashBang),
                "//?" => Some(Self::DoubleSlashQuestion),
                _ => None,
            };
            if let Some(operator) = operator {
                return Some((3, operator));
            }
        }
        let special = src3.map(|val| val == "==>").unwrap_or(false);
        if !special {
            let operator = src.get(..2).and_then(|operator| match operator {
                "==" => Some(Self::DoubleEqual),
                "!=" => Some(Self::NotEqual),
                "::" => Some(Self::DoubleColon),
                ".." => Some(Self::DoubleDot),
                ".<" => Some(Self::DotLess),
                ">." => Some(Self::GreaterDot),
                "><" => Some(Self::GreaterLess),
                "/!" => Some(Self::SlashBang),
                "/?" => Some(Self::SlashQuestion),
                "//" => Some(Self::DoubleSlash),
                "%!" => Some(Self::PercentBang),
                "%?" => Some(Self::PercentQuestion),
                "&&" => Some(Self::DoubleAmp),
                "||" => Some(Self::DoublePipe),
                ">=" => Some(Self::GreaterEqual),
                "<=" => Some(Self::LessEqual),
                "<-" => Some(Self::LeftArrow),
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
            ">" => Self::Greater,
            "<" => Self::Less,
            "?" => Self::Question,
            _ => return None,
        };
        Some((1, operator))
    }
}
