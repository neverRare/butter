#![warn(clippy::all)]

pub mod lexer;
pub mod parser;
pub mod span;
pub mod tree_vec;
pub mod tree;

#[macro_export]
macro_rules! assert_iter {
    ($iter:expr, $($item:expr),* $(,)?) => {{
        use std::iter::Iterator;
        use std::option::Option;
        let mut iter = $iter;
        $(
            std::assert_eq!(Iterator::next(&mut iter), Option::Some($item));
        )*
        std::assert_eq!(Iterator::next(&mut iter), Option::None);
    }};
}
