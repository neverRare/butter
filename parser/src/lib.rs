#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![forbid(unsafe_code)]

use crate::parser::insignificants;
use crate::statement::Statement;
use combine::eof;
use combine::many;
use combine::ParseError;
use combine::RangeStream;

pub mod expr;
mod parser;
pub mod pattern;
pub mod statement;

combine::parser! {
    pub fn ast['a, I]()(I) -> Vec<Statement<'a>>
    where [
        I: RangeStream<Token = char, Range = &'a str>,
        I::Error: ParseError<I::Token, I::Range, I::Position>,
    ] {
        insignificants().with(many(parser::statement::statement())).skip(eof())
    }
}
