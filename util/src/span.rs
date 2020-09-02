pub trait Span<'a>: Sized {
    fn from_range(src: &'a str, start: usize, end: usize) -> Self;
    fn start_on(&self, src: &'a str) -> usize;
    fn end_on(&self, src: &'a str) -> usize;
    fn span_len(&self) -> usize;
    fn from_spans(src: &'a str, start: &Self, end: &Self) -> Self {
        let start = start.start_on(src);
        let end = end.end_on(src);
        Self::from_range(src, start, end)
    }
}
impl<'a> Span<'a> for &'a str {
    fn from_range(src: &'a str, start: usize, end: usize) -> Self {
        &src[start..end]
    }
    fn start_on(&self, src: &'a str) -> usize {
        self.as_ptr() as usize - src.as_ptr() as usize
    }
    fn end_on(&self, src: &'a str) -> usize {
        self.start_on(src) + self.len()
    }
    fn span_len(&self) -> usize {
        self.len()
    }
}
