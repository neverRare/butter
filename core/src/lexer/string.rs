use std::num::NonZeroUsize;
use util::lexer::Lex;

pub enum Str<'a> {
    Str(&'a str),
    Char(&'a str),
    Unterminated,
}
impl<'a> Lex<'a> for Str<'a> {
    fn lex_first(src: &'a str) -> Option<(NonZeroUsize, Self)> {
        let mut chars = src.char_indices();
        let (_, first) = chars.next().unwrap();
        if let '\'' | '"' = first {
            let mut escaping = false;
            for (i, ch) in chars {
                match ch {
                    '\n' => return Some((NonZeroUsize::new(i + 1).unwrap(), Self::Unterminated)),
                    _ if escaping => escaping = false,
                    '\\' => escaping = true,
                    ch if ch == first => {
                        let content = &src[1..i];
                        let token = match first {
                            '\'' => Self::Char(content),
                            '"' => Self::Str(content),
                            _ => unreachable!(),
                        };
                        return Some((NonZeroUsize::new(i + 1).unwrap(), token));
                    }
                    _ => {}
                }
            }
            Some((NonZeroUsize::new(src.len()).unwrap(), Self::Unterminated))
        } else {
            None
        }
    }
}
