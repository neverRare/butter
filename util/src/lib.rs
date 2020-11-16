#![warn(clippy::all)]

pub mod iter;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod tree_vec;

pub fn aggregate_error<T, U, E, A>(left: Result<T, E>, right: Result<U, E>) -> Result<(T, U), E>
where
    E: Extend<A> + IntoIterator<Item = A>,
{
    match (left, right) {
        (Ok(left), Ok(right)) => Ok((left, right)),
        (Err(mut left), Err(right)) => {
            left.extend(right);
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
