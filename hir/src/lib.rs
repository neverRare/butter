#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![forbid(unsafe_code)]

use std::{collections::HashSet, fmt::Debug, hash::Hash};

pub mod expr;
pub mod pattern;
pub mod statement;

#[doc(hidden)]
pub mod hir_string_cache {
    include!(concat!(env!("OUT_DIR"), "/hir_string_cache.rs"));
}

pub use hir_string_cache::Atom;
use pretty::BoxDoc;

pub trait PrettyPrintType {
    const TYPED: bool;
    type FunScheme: PrettyPrintFunScheme + Debug + PartialEq + Eq + Clone;
    fn to_doc(&self) -> Option<BoxDoc>;
}
pub trait PrettyPrintFunScheme {
    fn to_doc(&self) -> Box<[BoxDoc]>;
}
impl PrettyPrintType for () {
    const TYPED: bool = false;
    type FunScheme = ();
    fn to_doc(&self) -> Option<BoxDoc> {
        None
    }
}
impl PrettyPrintFunScheme for () {
    fn to_doc(&self) -> Box<[BoxDoc]> {
        vec![].into()
    }
}
pub trait TraverseType {
    type Type: PrettyPrintType;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E>;
}
impl<T: TraverseType> TraverseType for Option<T> {
    type Type = T::Type;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        match self {
            Some(traverse) => traverse.traverse_type(data, for_type, for_scheme)?,
            None => (),
        }
        Ok(())
    }
}
pub fn bracket<'a>(left: &'a str, right: &'a str, content: BoxDoc<'a>) -> BoxDoc<'a> {
    BoxDoc::concat([
        BoxDoc::text(left),
        BoxDoc::concat([BoxDoc::line_(), content]).group().nest(4),
        BoxDoc::line_(),
        BoxDoc::text(right),
    ])
    .group()
}
pub fn intersperse_with_space<'a>(contents: impl IntoIterator<Item = BoxDoc<'a>>) -> BoxDoc<'a> {
    BoxDoc::intersperse(contents, BoxDoc::space()).group()
}
pub fn intersperse_with_line<'a>(contents: impl IntoIterator<Item = BoxDoc<'a>>) -> BoxDoc<'a> {
    BoxDoc::intersperse(contents, BoxDoc::line()).group()
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
