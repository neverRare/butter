use util::lexer::Lex;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Sign {
    Plus,
    Minus,
}
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Float<'a> {
    pub whole: &'a str,
    pub decimal: &'a str,
    pub mantissa_sign: Sign,
    pub mantissa: &'a str,
}
impl<'a> Lex<'a> for Float<'a> {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        todo!();
    }
}
