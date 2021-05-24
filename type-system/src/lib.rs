use crate::cons::Cons;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::iter::once;

mod cons;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct Var<'a> {
    name: &'a str,
    id: u32,
}
#[derive(Debug, PartialEq, Eq, Clone)]
enum Type<'a> {
    Var(Var<'a>),
    Cons(Cons<'a>),
}
impl<'a> Type<'a> {
    fn free_vars(&self) -> HashSet<Var<'a>> {
        match self {
            Self::Var(var) => once(*var).collect(),
            Self::Cons(cons) => cons.free_vars(),
        }
    }
    fn substitute(&mut self, subs: &Subs<'a>) {
        match self {
            Self::Var(var) => {
                if let Some(ty) = subs.get(var) {
                    *self = ty.clone()
                }
            }
            Self::Cons(cons) => cons.substitute(subs),
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
struct Scheme<'a> {
    for_all: HashSet<Var<'a>>,
    ty: Type<'a>,
}
impl<'a> Scheme<'a> {
    fn free_vars(&self) -> HashSet<Var<'a>> {
        self.ty
            .free_vars()
            .into_iter()
            .filter(|var| !self.for_all.contains(var))
            .collect()
    }
    fn substitute(&mut self, subs: &Subs<'a>) {
        let subs = subs
            .iter()
            .filter_map(|(var, ty)| {
                if self.for_all.contains(var) {
                    None
                } else {
                    Some((*var, ty.clone()))
                }
            })
            .collect();
        self.ty.substitute(&subs)
    }
}
type Subs<'a> = HashMap<Var<'a>, Type<'a>>;
struct Env<'a>(HashMap<Var<'a>, Scheme<'a>>);
impl<'a> Env<'a> {
    fn hashmap(&self) -> &HashMap<Var<'a>, Scheme<'a>> {
        let Self(map) = self;
        map
    }
    fn hashmap_mut(&mut self) -> &mut HashMap<Var<'a>, Scheme<'a>> {
        let Self(map) = self;
        map
    }
    fn remove(&mut self, var: Var<'a>) {
        self.hashmap_mut().remove(&var);
    }
    fn free_vars(&self) -> HashSet<Var<'a>> {
        self.hashmap()
            .values()
            .flat_map(Scheme::free_vars)
            .collect()
    }
    fn substitute(&mut self, subs: &Subs<'a>) {
        for (_, ty) in self.hashmap_mut() {
            ty.substitute(subs);
        }
    }
    fn generalize(&self, ty: Type<'a>) -> Scheme<'a> {
        let env_free_vars = self.free_vars();
        let for_all = ty
            .free_vars()
            .into_iter()
            .filter(|var| !env_free_vars.contains(var))
            .collect();
        Scheme { for_all, ty }
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
        fmt_intersperse(fmt, &self.for_all, ", ", Var::fmt)?;
        write!(fmt, ">")?;
        self.ty.fmt(fmt)?;
        Ok(())
    }
}
// replace this with `Iterator::intersperse` after
// https://github.com/rust-lang/rust/issues/79524 is resolved
fn fmt_intersperse<I>(
    fmt: &mut Formatter,
    iter: I,
    intersperse: &'static str,
    mut fmt_mapper: impl FnMut(I::Item, &mut Formatter) -> FmtResult,
) -> FmtResult
where
    I: IntoIterator,
{
    let mut iter = iter.into_iter().peekable();
    while let Some(item) = iter.next() {
        fmt_mapper(item, fmt)?;
        if iter.peek().is_some() {
            intersperse.fmt(fmt)?;
        }
    }
    Ok(())
}
