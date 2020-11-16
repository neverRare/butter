use util::lexer::Lex;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Radix {
    Hex,
    Dec,
    Oct,
    Bin,
}
impl Radix {
    fn from_indicator(src: &str) -> Option<Self> {
        match src {
            "0x" | "0X" => Some(Self::Hex),
            "0o" | "0O" => Some(Self::Oct),
            "0b" | "0B" => Some(Self::Bin),
            _ => None,
        }
    }
    fn valid_digit(self, ch: char) -> bool {
        match self {
            Self::Hex => matches!(ch, '_' | '0'..='9' | 'a'..='f' | 'A'..='F'),
            Self::Dec => matches!(ch, '_' | '0'..='9'),
            Self::Oct => matches!(ch, '_' | '0'..='7'),
            Self::Bin => matches!(ch, '_' | '0' | '1'),
        }
    }
}
pub struct Int<'a>(pub Radix, pub &'a str);
impl<'a> Lex<'a> for Int<'a> {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        let (radix, pos) = if let Some(radix) = src.get(..2).and_then(Radix::from_indicator) {
            (radix, 2)
        } else {
            (Radix::Dec, 0)
        };
        let content = &src[pos..];
        for (ind, ch) in content.char_indices() {
            if ch != '_' || !ch.is_alphanumeric() {
                let step = ind + pos;
                return if step != 0 {
                    Some((step, Self(radix, &content[..ind])))
                } else {
                    None
                };
            } else if !radix.valid_digit(ch) {
                return None;
            }
        }
        Some((src.len(), Self(radix, content)))
    }
}
