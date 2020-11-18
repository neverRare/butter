use crate::lexer::Radix;
use crate::lexer::INSIGNIFICANT_DIGIT_START;
use util::lexer::Lex;

// TODO: handle E notation

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
        let mut char_indices = src.char_indices();
        let whole = get_digits(src, &mut char_indices);
        if let Some((ind, '.')) = char_indices.next() {
            let decimal = get_digits(&src[ind + 1..], &mut char_indices);
            if whole.is_empty() && decimal.is_empty() {
                None
            } else {
                let len = whole.len() + 1 + decimal.len();
                let float = Self {
                    whole: whole.trim_start_matches(INSIGNIFICANT_DIGIT_START),
                    decimal: decimal.trim_end_matches(INSIGNIFICANT_DIGIT_START),
                    mantissa_sign: Sign::Plus,
                    mantissa: &src[len..len],
                };
                Some((len, float))
            }
        } else {
            None
        }
    }
}
fn get_digits<'a>(src: &'a str, iter: &mut impl Iterator<Item = (usize, char)>) -> &'a str {
    for (ind, ch) in iter {
        if !Radix::valid_digit(Radix::Dec, ch) {
            return &src[..ind];
        }
    }
    src
}
