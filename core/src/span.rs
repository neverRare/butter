pub use display_span::DisplaySpan;
pub mod display_span;

pub trait ExplainSpan {
    fn explain(&self) -> (&str, Option<&str>);
}
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
        Self::new(src, self.note, delta + self.from, delta + self.to)
    }
    pub fn map<U>(self, mapper: impl FnOnce(T) -> U) -> Span<'a, U> {
        Span {
            src: self.src,
            note: mapper(self.note),
            from: self.from,
            to: self.to,
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
    fn main_span(&self) -> (usize, usize) {
        (self.from, self.to)
    }
    fn spans(&self) -> Vec<(usize, usize)> {
        vec![]
    }
}
pub struct Spans<'a, T, U> {
    summary: T,
    src: &'a str,
    spans: Vec<(U, usize, usize)>,
}
impl<'a, T, U> Spans<'a, T, U> {
    pub fn new(summary: T, src: &'a str, spans: Vec<Span<'a, U>>) -> Self {
        let mut result = vec![];
        for span in spans {
            let Span {
                src: span_src,
                note,
                from,
                to,
            } = span;
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
