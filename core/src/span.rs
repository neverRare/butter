pub use display_span::DisplaySpan;
use std::ops::Range;
pub mod display_span;

#[derive(Debug, PartialEq, Eq)]
pub struct Span<'a> {
    src: &'a str,
    range: Range<usize>,
}
impl<'a> Span<'a> {
    pub fn new(src: &'a str, range: Range<usize>) -> Self {
        Self { src, range }
    }
    pub fn fit_from(&self, src: &'a str) -> Self {
        let inside = self.src.as_ptr() as usize;
        let outside = src.as_ptr() as usize;
        assert!(inside >= outside && inside + self.src.len() <= outside + src.len());
        let delta = inside - outside;
        let range = &self.range;
        Self::new(src, delta + range.start..delta + range.end)
    }
}
