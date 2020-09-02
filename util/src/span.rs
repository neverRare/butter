use std::ops::Range;

pub trait Span<'a>: Sized {
    fn from_range(src: &'a str, range: Range<usize>) -> Self;
    fn range_on(&self, src: &'a str) -> Range<usize>;
    fn from_spans(src: &'a str, start: &Self, end: &Self) -> Self {
        let start = start.range_on(src).start;
        let end = end.range_on(src).end;
        Self::from_range(src, start..end)
    }
}
impl<'a> Span<'a> for &'a str {
    fn from_range(src: &'a str, range: Range<usize>) -> Self {
        &src[range]
    }
    fn range_on(&self, src: &'a str) -> Range<usize> {
        let start = self.as_ptr() as usize - src.as_ptr() as usize;
        start..start + self.len()
    }
}
