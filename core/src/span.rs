#[derive(Debug, PartialEq, Eq)]
pub struct Span<'a, T> {
    src: &'a str,
    pub note: T,
    from: usize,
    to: usize,
}
impl<'a, T> Span<'a, T> {
    pub fn new(src: &'a str, note: T, from: usize, to: usize) -> Self {
        Self {
            src,
            note,
            from,
            to,
        }
    }
    pub fn fit_from(self, src: &'a str) -> Self {
        let inside = self.src.as_ptr() as usize;
        let outside = src.as_ptr() as usize;
        assert!(inside >= outside && inside + self.src.len() <= outside + src.len());
        let delta = inside - outside;
        Self {
            src,
            note: self.note,
            from: delta + self.from,
            to: delta + self.to,
        }
    }
}
pub struct Spans<'a, T, U> {
    summary: T,
    src: &'a str,
    spans: Vec<(U, usize, usize)>,
}
impl<'a, T, U> Spans<'a, T, U> {
    pub fn new(summary: T, src: &'a str, spans: impl IntoIterator<Item = Span<'a, U>>) -> Self {
        let result = vec![];
        for Span{src: span_src, note, from, to} in spans {
            assert_eq!(src, span_src);
            result.push((note, from, to));
        }
        Self {
            summary,
            src,
            spans: result,
        }
    }
}
