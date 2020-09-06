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
                if let '\n' = ch {
                    return Some((i + 1, Self::Unterminated));
                } else if escaping {
                    escaping = false;
                    continue;
                } else if let '\\' = ch {
                    escaping = true;
                    continue;
                } else if first == ch {
                    let content = &src[1..i];
                    let token = match first {
                        '\'' => Self::Char(content),
                        '"' => Self::Str(content),
                        _ => unreachable!(),
                    };
                    return Some((i + 1, token));
                }
            }
            Some((src.len(), Self::Unterminated))
        } else {
            None
        }
    }
}
