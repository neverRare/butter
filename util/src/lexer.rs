use std::iter::Filter;
use std::marker::PhantomData;

pub trait Lex<'a>: Sized {
    fn lex_first(src: &'a str) -> Option<(usize, Self)>;
    fn lex_first_span(src: &'a str) -> Option<(&'a str, Self)> {
        Self::lex_first(src).map(|(step, token)| (&src[..step], token))
    }
    fn lex(src: &'a str) -> Lexer<'a, Self> {
        Lexer::new(src)
    }
    fn lex_span(src: &'a str) -> SpanLexer<'a, Self> {
        SpanLexer::new(src)
    }
}
pub type FilterIter<'a, T> = Filter<Lexer<'a, T>, fn(&T) -> bool>;
pub type SpanFilterIter<'a, T> = Filter<SpanLexer<'a, T>, fn(&(&'a str, T)) -> bool>;
pub trait LexFilter<'a>: Lex<'a> {
    fn significant(&self) -> bool;
    fn lex(src: &'a str) -> FilterIter<'a, Self> {
        <Self as Lex>::lex(src).filter(Self::significant)
    }
    fn lex_span(src: &'a str) -> SpanFilterIter<'a, Self> {
        <Self as Lex>::lex_span(src).filter(|(_, token)| token.significant())
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
pub struct SpanLexer<'a, T> {
    src: &'a str,
    _phantom: PhantomData<fn() -> T>,
}
impl<'a, T> SpanLexer<'a, T> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            _phantom: PhantomData,
        }
    }
}
impl<'a, T> Iterator for SpanLexer<'a, T>
where
    T: Lex<'a>,
{
    type Item = (&'a str, T);
    fn next(&mut self) -> Option<Self::Item> {
        if self.src.is_empty() {
            None
        } else {
            match T::lex_first(self.src) {
                None => None,
                Some((step, token)) => {
                    let span = &self.src[..step];
                    self.src = &self.src[step..];
                    Some((span, token))
                }
            }
        }
    }
}
#[macro_export]
macro_rules! match_lex {
    ($src:expr; $($pat:pat => $expr:expr,)* => else $last_pat:pat => $last_expr:expr $(,)?) => {{
        let src = $src;
        $(
            if let Some((step, $pat)) = $crate::lexer::Lex::lex_first(src) {
                Some((step, $expr))
            } else
        )* {
            let $last_pat = src;
            $last_expr
        }
    }};
}
