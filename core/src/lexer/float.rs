use crate::lexer::Radix;
use crate::lexer::INSIGNIFICANT_DIGIT_START;
use std::num::NonZeroUsize;
use util::lexer::Lex;

// TODO #6: handle E notation

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Sign {
    Plus,
    Minus,
}
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Float<'a> {
    pub whole: &'a str,
    pub decimal: &'a str,
    pub exp_sign: Sign,
    pub exp: &'a str,
}
impl<'a> Lex<'a> for Float<'a> {
    fn lex_first(src: &'a str) -> Option<(NonZeroUsize, Self)> {
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
                    exp_sign: Sign::Plus,
                    exp: &src[len..len],
                };
                Some((NonZeroUsize::new(len).unwrap(), float))
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
