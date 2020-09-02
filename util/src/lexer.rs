use std::marker::PhantomData;

pub trait Lex<'a>: Sized {
    fn lex_first(src: &'a str) -> Option<(usize, Self)>;
    fn lex(src: &'a str) -> Lexer<'a, Self> {
        src.into()
    }
}
pub struct Lexer<'a, T> {
    src: &'a str,
    // is this okay?
    _phantom: PhantomData<fn() -> T>,
}
impl<'a, T> Lexer<'a, T> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            _phantom: PhantomData,
        }
    }
}
impl<'a, T> From<&'a str> for Lexer<'a, T> {
    fn from(src: &'a str) -> Self {
        Self::new(src)
    }
}
impl<'a, T> Iterator for Lexer<'a, T>
where
    T: Lex<'a>,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.src.is_empty() {
            None
        } else {
            match T::lex_first(self.src) {
                None => None,
                Some((step, token)) => {
                    self.src = &self.src[step..];
                    Some(token)
                }
            }
        }
    }
}
#[macro_export]
macro_rules! match_lex {
    ($src:expr; $($pat:pat => $expr:expr,)* else $last_pat:pat => $last_expr:expr $(,)?) => {{
        let src = $src;
        $(
            if let Some((step, $pat)) = $crate::Lex::lex_first(src) {
                Some(lex, $expr)
            } else
        )* {
            let $last_pat = src;
            $last_expr
        }
    }};
}
