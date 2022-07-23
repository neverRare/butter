#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![forbid(unsafe_code)]

use pretty_print::PrettyPrint;
use std::collections::HashSet;
use std::hash::Hash;

pub mod expr;
pub mod pattern;
pub mod pretty_print;
pub mod statement;

#[doc(hidden)]
pub mod hir_string_cache {
    include!(concat!(env!("OUT_DIR"), "/hir_string_cache.rs"));
}

pub use hir_string_cache::Atom;

pub trait PrettyType {
    type PrettyPrint: PrettyPrint + 'static;
    const TYPED: bool;
    fn pretty_print(&self) -> Option<Self::PrettyPrint>;
}
impl PrettyType for () {
    type PrettyPrint = String;
    const TYPED: bool = false;

    fn pretty_print(&self) -> Option<Self::PrettyPrint> {
        None
    }
}
fn all_unique<I>(iter: I) -> bool
where
    I: IntoIterator,
    I::Item: Clone + Hash + Eq,
{
    iter.into_iter()
        .try_fold(HashSet::new(), |mut set, item| {
            if set.contains(&item) {
                None
            } else {
                set.insert(item);
                Some(set)
            }
        })
        .is_some()
}
