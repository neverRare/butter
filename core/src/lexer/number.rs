use util::lexer::Lex;

pub struct Num<'a>(pub &'a str);
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
