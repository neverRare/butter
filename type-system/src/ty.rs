use crate::ty::cons::Cons;
use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display, Formatter},
    hash::Hash,
    iter::once,
};

pub mod cons;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Var<'a> {
    pub name: &'a str,
    pub id: u32,
}
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub(super) struct VarState<'a>(HashMap<&'a str, u32>);
impl<'a> VarState<'a> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn new_var(&mut self) -> Var<'a> {
        self.new_named("")
    }
    pub fn new_named(&mut self, name: &'a str) -> Var<'a> {
        let Self(map) = self;
        let state = map.entry(name).or_default();
        let id = *state;
        *state += 1;
        Var { name, id }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum Kind {
    Type,
    MutType,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct KindedVar<'a> {
    kind: Kind,
    var: Var<'a>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type<'a> {
    Var(Var<'a>),
    Cons(Cons<'a>),
}
impl<'a> Type<'a> {
    fn free_vars(&self) -> HashSet<KindedVar<'a>> {
        match self {
            Self::Var(var) => once(KindedVar {
                kind: Kind::Type,
                var: *var,
            })
            .collect(),
            Self::Cons(cons) => cons.free_vars(),
        }
    }
    pub(super) fn substitute(&mut self, subs: &Subs<'a>) -> Result<(), TypeError> {
        match self {
            Self::Var(var) => {
                if let Some(ty) = subs.get(*var) {
                    match ty {
                        Type1::Type(ty) => *self = ty.clone(),
                        Type1::MutType(_) => return Err(TypeError::MismatchKind),
                    }
                }
            }
            Self::Cons(cons) => cons.substitute(subs)?,
        }
        Ok(())
    }
    pub(super) fn unify_with(
        self,
        other: Self,
        var_state: &mut VarState<'a>,
    ) -> Result<Subs<'a>, TypeError> {
        let mut subs = Subs::new();
        match (self, other) {
            (Self::Cons(cons1), Self::Cons(cons2)) => {
                subs.compose_with(cons1.unify_with(cons2, var_state)?)?
            }
            (Self::Var(var), ty) | (ty, Self::Var(var)) => {
                if ty == Self::Var(var) {
                    // do nothing
                } else if ty.free_vars().contains(&KindedVar {
                    kind: Kind::Type,
                    var,
                }) {
                    return Err(TypeError::InfiniteOccurrence);
                } else {
                    subs.insert(var, Type1::Type(ty))
                }
            }
        }
        Ok(subs)
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MutType<'a> {
    Var(Var<'a>),
    Imm,
    Mut,
}
impl<'a> MutType<'a> {
    fn free_vars(&self) -> HashSet<KindedVar<'a>> {
        match self {
            Self::Var(var) => once(KindedVar {
                kind: Kind::MutType,
                var: *var,
            })
            .collect(),
            Self::Imm | Self::Mut => HashSet::new(),
        }
    }
    fn substitute(&mut self, subs: &Subs<'a>) -> Result<(), TypeError> {
        if let Self::Var(var) = self {
            if let Some(ty) = subs.get(*var) {
                match ty {
                    Type1::MutType(mutability) => *self = mutability,
                    Type1::Type(_) => return Err(TypeError::MismatchKind),
                }
            }
        }
        Ok(())
    }
    fn unify_with(self, other: Self) -> Result<Subs<'a>, TypeError> {
        let mut subs = Subs::new();
        match (self, other) {
            (Self::Mut, Self::Mut) | (Self::Imm, Self::Imm) => (),
            (Self::Var(var), ty) | (ty, Self::Var(var)) => {
                if ty == Self::Var(var) {
                    // do nothing
                } else if ty.free_vars().contains(&KindedVar {
                    kind: Kind::MutType,
                    var,
                }) {
                    return Err(TypeError::InfiniteOccurrence);
                } else {
                    subs.insert(var, Type1::MutType(ty))
                }
            }
            _ => return Err(TypeError::MismatchCons),
        }
        Ok(subs)
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
enum Type1<'a> {
    Type(Type<'a>),
    MutType(MutType<'a>),
}
impl<'a> From<KindedVar<'a>> for Type1<'a> {
    fn from(var: KindedVar<'a>) -> Self {
        match var.kind {
            Kind::Type => Self::Type(Type::Var(var.var)),
            Kind::MutType => Self::MutType(MutType::Var(var.var)),
        }
    }
}
impl<'a> Type1<'a> {
    fn kind(&self) -> Kind {
        match self {
            Self::Type(_) => Kind::Type,
            Self::MutType(_) => Kind::MutType,
        }
    }
    fn free_vars(&self) -> HashSet<KindedVar<'a>> {
        match self {
            Self::Type(ty) => ty.free_vars(),
            Self::MutType(ty) => ty.free_vars(),
        }
    }
    fn substitute(&mut self, subs: &Subs<'a>) -> Result<(), TypeError> {
        match self {
            Self::Type(ty) => ty.substitute(subs),
            Self::MutType(ty) => ty.substitute(subs),
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct Scheme<'a> {
    for_all: HashSet<KindedVar<'a>>,
    ty: Type<'a>,
}
impl<'a> Scheme<'a> {
    fn free_vars(&self) -> HashSet<KindedVar<'a>> {
        self.ty
            .free_vars()
            .into_iter()
            .filter(|var| !self.for_all.contains(var))
            .collect()
    }
    fn substitute(&mut self, subs: &Subs<'a>) -> Result<(), TypeError> {
        let mut subs = subs.clone();
        subs.filter_off(&self.for_all);
        self.ty.substitute(&subs)?;
        Ok(())
    }
    pub fn instantiate(self, var_state: &mut VarState<'a>) -> Result<Type<'a>, TypeError> {
        let subs = self
            .for_all
            .into_iter()
            .map(|var| {
                let new_var = var_state.new_named(var.var.name);
                (
                    var.var,
                    match var.kind {
                        Kind::Type => Type1::Type(Type::Var(new_var)),
                        Kind::MutType => Type1::MutType(MutType::Var(new_var)),
                    },
                )
            })
            .collect();
        let mut ty = self.ty;
        ty.substitute(&subs)?;
        Ok(ty)
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub(super) struct Subs<'a>(HashMap<Var<'a>, Type1<'a>>);
impl<'a> Subs<'a> {
    pub fn new() -> Self {
        Self::default()
    }
    fn hashmap(&self) -> &HashMap<Var<'a>, Type1<'a>> {
        let Self(map) = self;
        map
    }
    fn hashmap_mut(&mut self) -> &mut HashMap<Var<'a>, Type1<'a>> {
        let Self(map) = self;
        map
    }
    fn into_hashmap(self) -> HashMap<Var<'a>, Type1<'a>> {
        let Self(map) = self;
        map
    }
    fn get(&self, var: Var<'a>) -> Option<Type1<'a>> {
        self.hashmap().get(&var).map(Type1::clone)
    }
    fn insert(&mut self, var: Var<'a>, ty: Type1<'a>) {
        self.hashmap_mut().insert(var, ty);
    }
    fn filter_off(&mut self, vars: &HashSet<KindedVar<'a>>) {
        for var in vars {
            self.hashmap_mut().remove(&var.var);
        }
    }
    pub fn compose_with(&mut self, other: Self) -> Result<(), TypeError> {
        let map = self.hashmap_mut();
        for (_, ty) in map.iter_mut() {
            ty.substitute(&other)?;
        }
        map.extend(other.into_hashmap());
        Ok(())
    }
}
impl<'a> FromIterator<(Var<'a>, Type1<'a>)> for Subs<'a> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Var<'a>, Type1<'a>)>,
    {
        Self(iter.into_iter().collect())
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub(super) struct Env<'a>(HashMap<Var<'a>, Scheme<'a>>);
impl<'a> Env<'a> {
    pub fn new() -> Self {
        Self::default()
    }
    fn hashmap(&self) -> &HashMap<Var<'a>, Scheme<'a>> {
        let Self(map) = self;
        map
    }
    fn hashmap_mut(&mut self) -> &mut HashMap<Var<'a>, Scheme<'a>> {
        let Self(map) = self;
        map
    }
    pub fn get(&self, var: Var<'a>) -> Option<Scheme<'a>> {
        self.hashmap().get(&var).map(Scheme::clone)
    }
    fn remove(&mut self, var: Var<'a>) {
        self.hashmap_mut().remove(&var);
    }
    fn free_vars(&self) -> HashSet<KindedVar<'a>> {
        self.hashmap()
            .values()
            .flat_map(Scheme::free_vars)
            .collect()
    }
    fn substitute(&mut self, subs: &Subs<'a>) -> Result<(), TypeError> {
        for (_, ty) in self.hashmap_mut() {
            ty.substitute(subs)?;
        }
        Ok(())
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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TypeError {
    MismatchCons,
    MismatchKind,
    MismatchArity,
    InfiniteOccurrence,
    Overlap,
    UnboundVar,
}
impl Display for TypeError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        "Type Error".fmt(fmt)
    }
}
impl<'a> Display for Var<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}#{}", self.name, self.id)
    }
}
impl<'a> Display for KindedVar<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        self.var.fmt(fmt)
    }
}
impl<'a> Display for Type<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match &self {
            Type::Var(var) => var.fmt(fmt),
            Type::Cons(cons) => cons.fmt(fmt),
        }
    }
}
impl<'a> Display for MutType<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match &self {
            Self::Var(var) => var.fmt(fmt),
            Self::Imm => write!(fmt, "imm"),
            Self::Mut => write!(fmt, "mut"),
        }
    }
}
impl<'a> Display for Scheme<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "<")?;
        fmt_intersperse(fmt, &self.for_all, ", ", KindedVar::fmt)?;
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
    mut fmt_mapper: impl FnMut(I::Item, &mut Formatter) -> fmt::Result,
) -> fmt::Result
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
