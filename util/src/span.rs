// TODO: better debug implementation
#[derive(Clone, Copy, Debug)]
struct WholeSpan<'a> {
    src: &'a str,
    start: usize,
    end: usize,
}
impl<'a> PartialEq for WholeSpan<'a> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.src, other.src) && self.start == other.start && self.end == other.end
    }
}
impl<'a> Eq for WholeSpan<'a> {}
impl<'a> WholeSpan<'a> {
    fn span(self) -> &'a str {
        unsafe { self.src.get_unchecked(self.start..self.end) }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SpanOrEof<'a> {
    Span(WholeSpan<'a>),
    Eof,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Span<'a>(SpanOrEof<'a>);
impl<'a> Span<'a> {
    pub fn eof() -> Self {
        Self(SpanOrEof::Eof)
    }
    pub fn from_str(src: &'a str, span: &'a str) -> Self {
        assert!(
            src.as_ptr() <= span.as_ptr()
                && src.as_ptr() as usize + src.len() >= span.as_ptr() as usize + span.len()
        );
        let start = span.as_ptr() as usize - src.as_ptr() as usize;
        Self(SpanOrEof::Span(WholeSpan {
            src,
            start,
            end: start + span.len(),
        }))
    }
    pub fn up_to(self, end: Self) -> Self {
        let (src, start, end) = match (self.0, end.0) {
            (SpanOrEof::Eof, SpanOrEof::Eof) => return Self::eof(),
            (SpanOrEof::Eof, SpanOrEof::Span(end)) => {
                assert!(end.src.len() == end.start);
                (end.src, end.start, end.end)
            }
            (SpanOrEof::Span(start), SpanOrEof::Eof) => (start.src, start.start, start.src.len()),
            (SpanOrEof::Span(start), SpanOrEof::Span(end)) => {
                assert!(std::ptr::eq(start.src, end.src) && start.start < end.end);
                (start.src, start.start, end.end)
            }
        };
        let span = unsafe { src.get_unchecked(start..end) };
        Self::from_str(src, span)
    }
    pub fn src_and_span(self) -> Option<(&'a str, &'a str)> {
        match self.0 {
            SpanOrEof::Span(span) => Some((span.src, span.span())),
            SpanOrEof::Eof => None,
        }
    }
    pub fn span_with_src(self, src: &'a str) -> &'a str {
        match self.0 {
            SpanOrEof::Span(span) => {
                assert!(std::ptr::eq(span.src, src));
                span.span()
            }
            SpanOrEof::Eof => unsafe { src.get_unchecked(src.len()..) },
        }
    }
}
#[cfg(test)]
mod test {
    use crate::span::Span;
    #[test]
    fn from_into() {
        let src = "test";
        let span_src = &src[1..2];
        let eof_src = &src[4..];
        let span = Span::from_str(src, span_src);
        let eof = Span::eof();
        assert!(std::ptr::eq(span.span_with_src(src), span_src));
        assert!(std::ptr::eq(eof.span_with_src(src), eof_src));
    }
    #[test]
    fn up_to() {
        let src = "test";
        let left_src = &src[1..2];
        let right_src = &src[2..3];
        let result_src = &src[1..3];
        let left = Span::from_str(src, left_src);
        let right = Span::from_str(src, right_src);
        let result = Span::from_str(src, result_src);
        assert_eq!(left.up_to(right), result);
    }
}
