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
#[derive(Debug, PartialEq, Eq, Clone, Default)]
struct VarState<'a>(HashMap<&'a str, u32>);
impl<'a> VarState<'a> {
    fn new() -> Self {
        Self::default()
    }
    fn new_var(&mut self) -> Var<'a> {
        self.new_named("")
    }
    fn new_named(&mut self, name: &'a str) -> Var<'a> {
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
enum Type<'a> {
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
    fn substitute(&mut self, subs: &Subs<'a>) {
        match self {
            Self::Var(var) => {
                if let Some(ty) = subs.get(var) {
                    match ty {
                        Type1::Type(ty) => *self = ty.clone(),
                        Type1::MutType(_) => panic!("substituted mut type to type"),
                    }
                }
            }
            Self::Cons(cons) => cons.substitute(subs),
        }
    }
    fn unify_with(self, other: Self) -> Result<Subs<'a>, UnifyError> {
        match (self, other) {
            (Self::Cons(cons1), Self::Cons(cons2)) => cons1.unify_with(cons2),
            (Self::Var(var), ty) | (ty, Self::Var(var)) => {
                if ty == Self::Var(var) {
                    Ok(HashMap::new())
                } else if ty.free_vars().contains(&KindedVar {
                    kind: Kind::Type,
                    var,
                }) {
                    Err(UnifyError::InfiniteOccurrence)
                } else {
                    Ok(once((var, Type1::Type(ty))).collect())
                }
            }
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum MutType<'a> {
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
    fn substitute(&mut self, subs: &Subs<'a>) {
        if let Self::Var(var) = self {
            if let Some(ty) = subs.get(var) {
                match ty {
                    Type1::MutType(mutability) => *self = *mutability,
                    Type1::Type(_) => panic!("substituted type to mut type"),
                }
            }
        }
    }
    fn unify_with(self, other: Self) -> Result<Subs<'a>, UnifyError> {
        match (self, other) {
            (Self::Mut, Self::Mut) | (Self::Imm, Self::Imm) => Ok(HashMap::new()),
            (Self::Var(var), ty) | (ty, Self::Var(var)) => {
                if ty == Self::Var(var) {
                    Ok(HashMap::new())
                } else if ty.free_vars().contains(&KindedVar {
                    kind: Kind::MutType,
                    var,
                }) {
                    Err(UnifyError::InfiniteOccurrence)
                } else {
                    Ok(once((var, Type1::MutType(ty))).collect())
                }
            }
            _ => Err(UnifyError::MismatchCons),
        }
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
    fn substitute(&mut self, subs: &Subs<'a>) {
        match self {
            Self::Type(ty) => ty.substitute(subs),
            Self::MutType(ty) => ty.substitute(subs),
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
struct Scheme<'a> {
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
    fn substitute(&mut self, subs: &Subs<'a>) {
        let subs = subs
            .iter()
            .filter_map(|(var, ty)| {
                if self.for_all.contains(&KindedVar {
                    kind: ty.kind(),
                    var: *var,
                }) {
                    None
                } else {
                    Some((*var, ty.clone()))
                }
            })
            .collect();
        self.ty.substitute(&subs);
    }
    fn instantiate(self, var_state: &mut VarState<'a>) -> Type<'a> {
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
        ty.substitute(&subs);
        ty
    }
}
type Subs<'a> = HashMap<Var<'a>, Type1<'a>>;
fn compose<'a>(subs: &mut Subs<'a>, more_subs: Subs<'a>) {
    for (_, ty) in subs.iter_mut() {
        ty.substitute(&more_subs);
    }
    subs.extend(more_subs);
}
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
    fn free_vars(&self) -> HashSet<KindedVar<'a>> {
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
enum UnifyError {
    MismatchCons,
    InfiniteOccurrence,
}
impl<'a> Display for Var<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{}#{}", self.name, self.id)
    }
}
impl<'a> Display for KindedVar<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        self.var.fmt(fmt)
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
impl<'a> Display for MutType<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match &self {
            Self::Var(var) => var.fmt(fmt),
            Self::Imm => write!(fmt, "imm"),
            Self::Mut => write!(fmt, "mut"),
        }
    }
}
impl<'a> Display for Scheme<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
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
