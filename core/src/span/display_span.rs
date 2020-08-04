use std::{
    io::{Error, Write},
    ops::Range,
};

pub trait DisplaySpan<'a> {
    fn src(&self) -> &'a str;
    fn summarize(&self) -> String;
    fn explain(&self) -> Option<String>;
    fn main_span(&self) -> &Range<usize>;
    fn spans(&self) -> Vec<&Range<usize>>;
    fn write(&self, write: &mut impl Write) -> Result<(), Error> {
        writeln!(write, "{}", self.summarize())?;
        todo!()
    }
}
