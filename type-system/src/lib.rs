use crate::cons::Cons;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

mod cons;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct Var<'a> {
    name: &'a str,
    id: u32,
}
#[derive(Debug, PartialEq, Eq, Clone)]
enum Type<'a> {
    Var(Var<'a>),
    Cons(Cons<'a>),
}
#[derive(Debug, PartialEq, Eq, Clone)]
struct Scheme<'a> {
    for_all: HashSet<Var<'a>>,
    ty: Type<'a>,
}
struct Subs<'a>(HashMap<Var<'a>, Type<'a>>);
impl<'a> Subs<'a> {
    fn hashmap(&self) -> &HashMap<Var<'a>, Type<'a>> {
        let Self(map) = self;
        map
    }
}
struct Env<'a>(HashMap<Var<'a>, Scheme<'a>>);
impl<'a> Env<'a> {
    fn hashmap(&self) -> &HashMap<Var<'a>, Scheme<'a>> {
        let Self(map) = self;
        map
    }
}
impl<'a> Display for Var<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{}#{}", self.name, self.id)
    }
}
impl<'a> Display for Type<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match &self {
            Type::Var(var) => var.fmt(fmt),
            Type::Cons(cons) => cons.fmt(fmt),
        }
    }
}
impl<'a> Display for Scheme<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "<")?;
        fmt_in_comma(fmt, &self.for_all, Var::fmt)?;
        write!(fmt, ">")?;
        self.ty.fmt(fmt)?;
        Ok(())
    }
}
fn fmt_in_comma<I>(
    fmt: &mut Formatter,
    iter: I,
    mut fmt_mapper: impl FnMut(I::Item, &mut Formatter) -> FmtResult,
) -> FmtResult
where
    I: IntoIterator,
{
    let mut iter = iter.into_iter().peekable();
    while let Some(item) = iter.next() {
        fmt_mapper(item, fmt)?;
        if iter.peek().is_some() {
            write!(fmt, ", ")?;
        }
    }
    Ok(())
}
