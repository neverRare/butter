use std::iter::Peekable;

pub trait PeekableIterator: Iterator {
    fn peek(&mut self) -> Option<&Self::Item>;
}
impl<T: Iterator> PeekableIterator for Peekable<T> {
    fn peek(&mut self) -> Option<&Self::Item> {
        Peekable::peek(self)
    }
}
