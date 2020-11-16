use util::lexer::Lex;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Radix {
    Hex,
    Dec,
    Oct,
    Bin,
}
pub struct Int<'a>(pub Radix, pub &'a str);
impl<'a> Lex<'a> for Int<'a> {
    fn lex_first(src: &'a str) -> Option<(usize, Self)> {
        todo!();
    }
}
