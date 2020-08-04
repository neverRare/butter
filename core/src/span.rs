pub use display_span::DisplaySpan;
use std::ops::Range;
pub mod display_span;

pub trait ExplainSpan {
    fn explain(&self) -> (&str, Option<&str>);
}
#[derive(Debug, PartialEq, Eq)]
pub struct Span<'a, T> {
    src: &'a str,
    pub note: T,
    range: Range<usize>,
}
impl<'a, T> Span<'a, T> {
    pub fn new(src: &'a str, note: T, range: Range<usize>) -> Self {
        Self { src, note, range }
    }
    pub fn fit_from(self, src: &'a str) -> Self {
        let inside = self.src.as_ptr() as usize;
        let outside = src.as_ptr() as usize;
        assert!(inside >= outside && inside + self.src.len() <= outside + src.len());
        let delta = inside - outside;
        let range = self.range;
        Self::new(src, self.note, delta + range.start..delta + range.end)
    }
    pub fn map<U>(self, mapper: impl FnOnce(T) -> U) -> Span<'a, U> {
        Span {
            src: self.src,
            note: mapper(self.note),
            range: self.range,
        }
    }
}
impl<'a, T> DisplaySpan<'a> for Span<'a, T>
where
    T: ExplainSpan,
{
    fn src(&self) -> &'a str {
        self.src
    }
    fn summarize(&self) -> String {
        self.note.explain().0.to_string()
    }
    fn explain(&self) -> Option<String> {
        self.note.explain().1.map(|val| val.to_string())
    }
    fn main_span(&self) -> &Range<usize> {
        &self.range
    }
    fn spans(&self) -> Vec<&Range<usize>> {
        vec![]
    }
}
