use crate::lexer::Radix;
use crate::lexer::INSIGNIFICANT_DIGIT_START;
use std::num::NonZeroUsize;
use util::iter::PeekableIterator;
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
    pub exp_sign: Sign,
    pub exp: &'a str,
}
impl<'a> Lex<'a> for Float<'a> {
    fn lex_first(src: &'a str) -> Option<(NonZeroUsize, Self)> {
        let mut char_indices = src.char_indices().peekable();
        let whole = get_digits(src, 0, &mut char_indices);
        let decimal = if let Some('.') = peek_char(&mut char_indices) {
            let (ind, _) = char_indices.next().unwrap();
            get_digits(&src, ind + 1, &mut char_indices)
        } else {
            &whole[whole.len()..]
        };
        if whole.is_empty() && decimal.is_empty() {
            return None;
        }
        let (exp_sign, exp) = if let Some('e') | Some('E') = peek_char(&mut char_indices) {
            let (ind, _) = char_indices.next().unwrap();
            let (pos, exp_sign) = match peek_char(&mut char_indices) {
                Some('+') => {
                    char_indices.next().unwrap();
                    (ind + 2, Sign::Plus)
                }
                Some('-') => {
                    char_indices.next().unwrap();
                    (ind + 2, Sign::Minus)
                }
                Some(_) | None => (ind + 1, Sign::Plus),
            };
            let exp = get_digits(src, pos, &mut char_indices);
            if exp.is_empty() {
                return None;
            }
            (exp_sign, exp)
        } else {
            (Sign::Plus, &decimal[decimal.len()..])
        };
        if decimal.is_empty() && exp.is_empty() {
            None
        } else {
            let len = match char_indices.next() {
                Some((ind, _)) => ind,
                None => src.len(),
            };
            Some((
                NonZeroUsize::new(len).unwrap(),
                Self {
                    whole: whole.trim_start_matches(INSIGNIFICANT_DIGIT_START),
                    decimal: decimal.trim_end_matches(INSIGNIFICANT_DIGIT_START),
                    exp_sign,
                    exp: exp.trim_start_matches(INSIGNIFICANT_DIGIT_START),
                },
            ))
        }
    }
}
fn get_digits<'a>(
    src: &'a str,
    start: usize,
    iter: &mut impl PeekableIterator<Item = (usize, char)>,
) -> &'a str {
    while let Some(&(ind, ch)) = iter.peek() {
        if Radix::valid_digit(Radix::Dec, ch) {
            iter.next();
        } else {
            return &src[start..ind];
        }
    }
    &src[start..src.len()]
}
fn peek_char(peekable_iter: &mut impl PeekableIterator<Item = (usize, char)>) -> Option<char> {
    peekable_iter.peek().map(|(_, ch)| *ch)
}
