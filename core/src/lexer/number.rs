use util::lexer::Lex;

pub struct Num;
impl<'a> Lex<'a> for Num {
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
                    return Some((i + 1, Self));
                }
            }
            Some((src.len(), Self))
        } else {
            None
        }
    }
}
