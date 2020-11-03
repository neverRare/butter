#![warn(clippy::all)]

pub mod lexer;
pub mod parser;
pub mod span;
pub mod tree_vec;

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
#[macro_export]
macro_rules! mini_fn {
    (($a:expr $(,)?); $($path:path,)* => else $else:expr $(,)?) => {{
        let a = $a;
        $(
            if let Some(val) = $path(a) {
                val
            } else
        )* {
            $else
        }
    }};
    (($a:expr, $b:expr $(,)?); $($path:path,)* => else $else:expr $(,)?) => {{
        let a = $a;
        let b = $b;
        $(
            if let Some(val) = $path(a, b) {
                val
            } else
        )* {
            $else
        }
    }};
    (($a:expr, $b:expr, $c: expr $(,)?); $($path:path,)* => else $else:expr $(,)?) => {{
        let a = $a;
        let b = $b;
        let c = $c;
        $(
            if let Some(val) = $path(a, b, c) {
                val
            } else
        )* {
            $else
        }
    }};
}
