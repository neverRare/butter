pub struct ErrorSpan<'a, T> {
    err: T,
    src: &'a str,
    from: usize,
    to: usize,
}
impl<'a, T> ErrorSpan<'a, T> {
    pub fn new(err: T, src: &'a str, from: usize, to: usize) -> Self {
        Self { err, src, from, to }
    }
}
