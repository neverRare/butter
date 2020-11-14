use util::lexer::Lex;

pub enum Str<'a> {
    Str(&'a str),
    Char(&'a str),
    Unterminated,
}
impl<'a> Lex<'a> for Str<'a> {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        let mut chars = src.char_indices();
        let (_, first) = chars.next().unwrap();
        if let '\'' | '"' = first {
            let mut escaping = false;
            for (i, ch) in chars {
                match ch {
                    '\n' => return Some((i + 1, Self::Unterminated)),
                    _ if escaping => escaping = false,
                    '\\' => escaping = true,
                    ch if ch == first => {
                        let content = &src[1..i];
                        let token = match first {
                            '\'' => Self::Char(content),
                            '"' => Self::Str(content),
                            _ => unsafe { std::hint::unreachable_unchecked() },
                        };
                        return Some((i + 1, token));
                    }
                    _ => {}
                }
            }
            Some((src.len(), Self::Unterminated))
        } else {
            None
        }
    }
}
