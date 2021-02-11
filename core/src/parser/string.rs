use crate::parser::error::ErrorType;
use crate::parser::Error;
use crate::parser::Node;
use crate::parser::NodeType;
use std::num::NonZeroUsize;
use util::lexer::Lex;
use util::match_lex;
use util::tree_vec::Tree;
use util::tree_vec::TreeVec;

// TODO unit tests

struct SimpleEscape(u8);
impl<'a> Lex<'a> for SimpleEscape {
    fn lex_first(src: &'a str) -> Option<(NonZeroUsize, Self)> {
        if &src[0..1] == "\\" && src.len() > 1 {
            let byte = match src.get(1..2)? {
                "\\" => b'\\',
                "\"" => b'"',
                "'" => b'\'',
                "n" => b'\n',
                "r" => b'\r',
                "t" => b'\t',
                "v" => 11,
                "0" => 0,
                _ => return None,
            };
            Some((NonZeroUsize::new(2).unwrap(), Self(byte)))
        } else {
            None
        }
    }
}
enum ByteEscape {
    Byte(u8),
    Invalid,
}
impl<'a> Lex<'a> for ByteEscape {
    fn lex_first(src: &'a str) -> Option<(NonZeroUsize, Self)> {
        if src.get(0..2) != Some("\\x") {
            return None;
        }
        if let Some(code) = src.get(2..4) {
            if let Some(i) = code.find('\\') {
                return Some((NonZeroUsize::new(2 + i).unwrap(), Self::Invalid));
            }
            let result = match <u8>::from_str_radix(code, 16) {
                Ok(byte) => Self::Byte(byte),
                Err(_) => Self::Invalid,
            };
            Some((NonZeroUsize::new(4).unwrap(), result))
        } else {
            None
        }
    }
}
struct Escape;
impl<'a> Lex<'a> for Escape {
    fn lex_first(src: &'a str) -> Option<(NonZeroUsize, Self)> {
        if src.get(0..1) == Some("\\") {
            let next_len = src[1..].chars().next().map(<char>::len_utf8).unwrap_or(0);
            Some((NonZeroUsize::new(1 + next_len).unwrap(), Self))
        } else {
            None
        }
    }
}
enum Char {
    Char(char),
    Byte(u8),
    Invalid,
}
impl<'a> Lex<'a> for Char {
    fn lex_first(src: &'a str) -> Option<(NonZeroUsize, Self)> {
        match_lex! { src;
            SimpleEscape(byte) => Self::Byte(byte),
            ByteEscape::Byte(byte) => Self::Byte(byte),
            ByteEscape::Invalid => Self::Invalid,
            Escape => Self::Invalid,
            => else src => {
                let ch = src.chars().next().unwrap();
                Some((NonZeroUsize::new(ch.len_utf8()).unwrap(), Self::Char(ch)))
            },
        }
    }
}
pub(super) fn parse_content(content: &str) -> Result<TreeVec<Node>, Vec<Error>> {
    let mut chars = TreeVec::new();
    let mut errors = vec![];
    for (span, ch) in Char::lex_span(content) {
        match ch {
            Char::Char(ch) => {
                if errors.is_empty() {
                    let len = ch.len_utf8();
                    let mut bytes = [0; 4];
                    ch.encode_utf8(&mut bytes);
                    chars.reserve(len);
                    for byte in bytes.iter().take(len) {
                        chars.push(Tree::new(Node {
                            span,
                            node: NodeType::CharInside(*byte),
                        }))
                    }
                }
            }

            Char::Byte(byte) => {
                if errors.is_empty() {
                    chars.push(Tree::new(Node {
                        span,
                        node: NodeType::CharInside(byte),
                    }))
                }
            }
            Char::Invalid => errors.push(Error {
                span,
                error: ErrorType::InvalidEscape,
            }),
        }
    }
    if errors.is_empty() {
        Ok(chars)
    } else {
        Err(errors)
    }
}
