#![warn(clippy::all)]

pub mod iter;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod tree_vec;

pub fn aggregate_error<T, U, E>(
    left: Result<T, Vec<E>>,
    right: Result<U, Vec<E>>,
) -> Result<(T, U), Vec<E>> {
    match (left, right) {
        (Ok(left), Ok(right)) => Ok((left, right)),
        (Err(mut left), Err(mut right)) => {
            left.append(&mut right);
            Err(left)
        }
        (Err(err), _) | (_, Err(err)) => Err(err),
    }
}
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
