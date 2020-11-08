#[derive(Clone, Copy)]
struct WholeSpan<'a> {
    src: &'a str,
    start: usize,
    end: usize,
}
#[derive(Clone, Copy)]
enum SpanOrEof<'a> {
    Span(WholeSpan<'a>),
    Eof,
}
#[derive(Clone, Copy)]
pub struct Span<'a>(SpanOrEof<'a>);
impl<'a> Span<'a> {
    pub fn eof() -> Self {
        Self(SpanOrEof::Eof)
    }
    pub fn from_str(src: &'a str, span: &'a str) -> Self {
        debug_assert!(
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
                assert!(std::ptr::eq(start.src, end.src));
                (start.src, start.start, end.end)
            }
        };
        Self::from_str(src, &src[start..end])
    }
    pub fn src_and_span(self) -> Option<(&'a str, &'a str)> {
        match self.0 {
            SpanOrEof::Span(span) => Some((span.src, &span.src[span.start..span.end])),
            SpanOrEof::Eof => None,
        }
    }
    pub fn span_with_src(self, src: &'a str) -> &'a str {
        match self.0 {
            SpanOrEof::Span(span) => {
                assert!(std::ptr::eq(span.src, src));
                &src[span.start..span.end]
            }
            SpanOrEof::Eof => &src[src.len()..],
        }
    }
}
