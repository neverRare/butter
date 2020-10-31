#[derive(Clone, Copy)]
pub struct Span<'a> {
    src: &'a str,
    start: usize,
    end: usize,
}
impl<'a> Span<'a> {
    pub fn from_str(src: &'a str, span: &'a str) -> Self {
        assert!(
            src.as_ptr() <= span.as_ptr()
                && src.as_ptr() as usize + src.len() >= span.as_ptr() as usize + span.len()
        );
        let start = span.as_ptr() as usize - src.as_ptr() as usize;
        Self {
            src,
            start,
            end: start + span.len(),
        }
    }
    pub fn up_to(self, end: Self) -> Self {
        assert!(std::ptr::eq(self.src, end.src));
        Self {
            src: self.src,
            start: self.start,
            end: end.end,
        }
    }
    pub fn span(self) -> &'a str {
        &self.src[self.start..self.end]
    }
}
