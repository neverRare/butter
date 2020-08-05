pub use display_span::DisplaySpan;
use std::ops::Range;
pub mod display_span;

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
}
