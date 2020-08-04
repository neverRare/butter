use std::io::{Error, Write};

pub trait DisplaySpan<'a> {
    fn src(&self) -> &'a str;
    fn summarize(&self) -> String;
    fn explain(&self) -> Option<String>;
    fn main_span(&self) -> (usize, usize);
    fn spans(&self) -> Vec<(usize, usize)>;
    fn write(&self, write: &mut impl Write) -> Result<(), Error> {
        writeln!(write, "{}", self.summarize());
        todo!()
    }
}
