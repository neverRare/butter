use std::iter::Peekable;

pub trait PeekableIter: Iterator {
    fn peek(&mut self) -> Option<&Self::Item>;
}
impl<T: Iterator> PeekableIter for Peekable<T> {
    fn peek(&mut self) -> Option<&Self::Item> {
        Peekable::peek(self)
    }
}
