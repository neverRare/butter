#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![forbid(unsafe_code)]

use pretty_print::PrettyPrintTree;
use std::{collections::HashSet, fmt::Debug, hash::Hash};

pub mod expr;
pub mod pattern;
pub mod pretty_print;
pub mod statement;

#[doc(hidden)]
pub mod hir_string_cache {
    include!(concat!(env!("OUT_DIR"), "/hir_string_cache.rs"));
}

pub use hir_string_cache::Atom;

pub trait PrettyPrintType {
    const TYPED: bool;
    type FunScheme: PrettyPrintFunScheme + Debug + PartialEq + Eq + Clone;
    fn to_pretty_print(&self) -> Option<Box<dyn PrettyPrintTree>>;
}
pub trait PrettyPrintFunScheme {
    fn to_pretty_print_generics(&self) -> Box<[Box<dyn PrettyPrintTree>]>;
}
impl PrettyPrintType for () {
    const TYPED: bool = false;
    type FunScheme = ();
    fn to_pretty_print(&self) -> Option<Box<dyn PrettyPrintTree>> {
        None
    }
}
impl PrettyPrintFunScheme for () {
    fn to_pretty_print_generics(&self) -> Box<[Box<dyn PrettyPrintTree>]> {
        vec![].into()
    }
}
pub trait TraverseType {
    type Type: PrettyPrintType;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: impl FnMut(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: impl FnMut(
            &mut <Self::Type as PrettyPrintType>::FunScheme,
            &mut U,
        ) -> Result<(), E>,
    ) -> Result<(), E>;
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
