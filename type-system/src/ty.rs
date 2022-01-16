use crate::ty::cons::Cons;
use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display, Formatter},
    hash::Hash,
    iter::once,
};
use string_cache::DefaultAtom;

pub mod cons;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Var {
    pub name: DefaultAtom,
    pub id: u32,
}
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub(super) struct VarState(HashMap<DefaultAtom, u32>);
impl VarState {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn new_var(&mut self) -> Var {
        self.new_named(&DefaultAtom::from(""))
    }
    pub fn new_named(&mut self, name: &DefaultAtom) -> Var {
        let Self(map) = self;
        let state = map.entry(name.clone()).or_default();
        let id = *state;
        *state += 1;
        Var {
            name: name.clone(),
            id,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum Kind {
    Type,
    MutType,
}
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct KindedVar {
    kind: Kind,
    var: Var,
}
trait FreeVars {
    fn free_vars(&self) -> HashSet<KindedVar>;
}
pub(super) trait Substitutable {
    fn substitute(&mut self, subs: &Subs) -> Result<(), TypeError>;
}
pub(super) trait Unifiable {
    fn unify_with(self, other: Self, var_state: &mut VarState) -> Result<Subs, TypeError>;
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    Var(Var),
    Cons(Cons),
}
impl FreeVars for Type {
    fn free_vars(&self) -> HashSet<KindedVar> {
        match self {
            Self::Var(var) => once(KindedVar {
                kind: Kind::Type,
                var: var.clone(),
            })
            .collect(),
            Self::Cons(cons) => cons.free_vars(),
        }
    }
}
impl Substitutable for Type {
    fn substitute(&mut self, subs: &Subs) -> Result<(), TypeError> {
        match self {
            Self::Var(var) => {
                if let Some(ty) = subs.get(var) {
                    match ty {
                        Type1::Type(ty) => *self = ty,
                        Type1::MutType(_) => return Err(TypeError::MismatchKind),
                    }
                }
            }
            Self::Cons(cons) => cons.substitute(subs)?,
        }
        Ok(())
    }
}
impl FreeVars for (DefaultAtom, Type) {
    fn free_vars(&self) -> HashSet<KindedVar> {
        let (_, ty) = self;
        ty.free_vars()
    }
}
impl Substitutable for (DefaultAtom, Type) {
    fn substitute(&mut self, subs: &Subs) -> Result<(), TypeError> {
        let (_, ty) = self;
        ty.substitute(subs)
    }
}
impl Unifiable for Type {
    fn unify_with(self, other: Self, var_state: &mut VarState) -> Result<Subs, TypeError> {
        let mut subs = Subs::new();
        match (self, other) {
            (Self::Cons(cons1), Self::Cons(cons2)) => {
                subs.compose_with(cons1.unify_with(cons2, var_state)?)?
            }
            (Self::Var(var), ty) | (ty, Self::Var(var)) => {
                if ty == Self::Var(var.clone()) {
                    // do nothing
                } else if ty.free_vars().contains(&KindedVar {
                    kind: Kind::Type,
                    var: var.clone(),
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
impl Unifiable for (DefaultAtom, Type) {
    fn unify_with(self, other: Self, var_state: &mut VarState) -> Result<Subs, TypeError> {
        let (name1, ty1) = self;
        let (name2, ty2) = other;
        if name1 != name2 {
            return Err(TypeError::MismatchName);
        }
        ty1.unify_with(ty2, var_state)
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MutType {
    Var(Var),
    Imm,
    Mut,
}
impl FreeVars for MutType {
    fn free_vars(&self) -> HashSet<KindedVar> {
        match self {
            Self::Var(var) => once(KindedVar {
                kind: Kind::MutType,
                var: var.clone(),
            })
            .collect(),
            Self::Imm | Self::Mut => HashSet::new(),
        }
    }
}
impl Substitutable for MutType {
    fn substitute(&mut self, subs: &Subs) -> Result<(), TypeError> {
        if let Self::Var(var) = self {
            if let Some(ty) = subs.get(var) {
                match ty {
                    Type1::MutType(mutability) => *self = mutability,
                    Type1::Type(_) => return Err(TypeError::MismatchKind),
                }
            }
        }
        Ok(())
    }
}
impl Unifiable for MutType {
    fn unify_with(self, other: Self, _: &mut VarState) -> Result<Subs, TypeError> {
        let mut subs = Subs::new();
        match (self, other) {
            (Self::Mut, Self::Mut) | (Self::Imm, Self::Imm) => (),
            (Self::Var(var), ty) | (ty, Self::Var(var)) => {
                if ty == Self::Var(var.clone()) {
                    // do nothing
                } else if ty.free_vars().contains(&KindedVar {
                    kind: Kind::MutType,
                    var: var.clone(),
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
enum Type1 {
    Type(Type),
    MutType(MutType),
}
impl From<KindedVar> for Type1 {
    fn from(var: KindedVar) -> Self {
        match var.kind {
            Kind::Type => Self::Type(Type::Var(var.var)),
            Kind::MutType => Self::MutType(MutType::Var(var.var)),
        }
    }
}
impl Type1 {
    fn kind(&self) -> Kind {
        match self {
            Self::Type(_) => Kind::Type,
            Self::MutType(_) => Kind::MutType,
        }
    }
}
impl FreeVars for Type1 {
    fn free_vars(&self) -> HashSet<KindedVar> {
        match self {
            Self::Type(ty) => ty.free_vars(),
            Self::MutType(ty) => ty.free_vars(),
        }
    }
}
impl Substitutable for Type1 {
    fn substitute(&mut self, subs: &Subs) -> Result<(), TypeError> {
        match self {
            Self::Type(ty) => ty.substitute(subs),
            Self::MutType(ty) => ty.substitute(subs),
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct Scheme {
    for_all: HashSet<KindedVar>,
    ty: Type,
}
impl FreeVars for Scheme {
    fn free_vars(&self) -> HashSet<KindedVar> {
        self.ty
            .free_vars()
            .into_iter()
            .filter(|var| !self.for_all.contains(var))
            .collect()
    }
}
impl Substitutable for Scheme {
    fn substitute(&mut self, subs: &Subs) -> Result<(), TypeError> {
        let mut subs = subs.clone();
        subs.filter_off(&self.for_all);
        self.ty.substitute(&subs)?;
        Ok(())
    }
}
impl Scheme {
    pub fn instantiate(self, var_state: &mut VarState) -> Result<Type, TypeError> {
        let subs = self
            .for_all
            .into_iter()
            .map(|var| {
                let new_var = var_state.new_named(&var.var.name);
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
pub(super) struct Subs(HashMap<Var, Type1>);
impl Subs {
    pub fn new() -> Self {
        Self::default()
    }
    fn hashmap(&self) -> &HashMap<Var, Type1> {
        let Self(map) = self;
        map
    }
    fn hashmap_mut(&mut self) -> &mut HashMap<Var, Type1> {
        let Self(map) = self;
        map
    }
    fn into_hashmap(self) -> HashMap<Var, Type1> {
        let Self(map) = self;
        map
    }
    fn get(&self, var: &Var) -> Option<Type1> {
        self.hashmap().get(var).map(Type1::clone)
    }
    fn insert(&mut self, var: Var, ty: Type1) {
        self.hashmap_mut().insert(var, ty);
    }
    fn filter_off(&mut self, vars: &HashSet<KindedVar>) {
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
impl FromIterator<(Var, Type1)> for Subs {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Var, Type1)>,
    {
        Self(iter.into_iter().collect())
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub(super) struct Env(HashMap<Var, Scheme>);
impl Env {
    pub fn new() -> Self {
        Self::default()
    }
    fn hashmap(&self) -> &HashMap<Var, Scheme> {
        let Self(map) = self;
        map
    }
    fn hashmap_mut(&mut self) -> &mut HashMap<Var, Scheme> {
        let Self(map) = self;
        map
    }
    pub fn get(&self, var: &Var) -> Option<Scheme> {
        self.hashmap().get(var).map(Scheme::clone)
    }
    fn remove(&mut self, var: Var) {
        self.hashmap_mut().remove(&var);
    }
    fn free_vars(&self) -> HashSet<KindedVar> {
        self.hashmap()
            .values()
            .flat_map(Scheme::free_vars)
            .collect()
    }
    fn substitute(&mut self, subs: &Subs) -> Result<(), TypeError> {
        for ty in self.hashmap_mut().values_mut() {
            ty.substitute(subs)?;
        }
        Ok(())
    }
    fn generalize(&self, ty: Type) -> Scheme {
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
    MismatchName,
    InfiniteOccurrence,
    Overlap,
    UnboundVar,
}
impl Display for TypeError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        write!(fmt, "TypeError")
    }
}
impl Display for Var {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}#{}", self.name, self.id)
    }
}
impl Display for KindedVar {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}", self.var)
    }
}
impl Display for Type {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match &self {
            Type::Var(var) => write!(fmt, "{}", var),
            Type::Cons(cons) => write!(fmt, "{}", cons),
        }
    }
}
impl Display for MutType {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match &self {
            Self::Var(var) => write!(fmt, "{}", var),
            Self::Imm => write!(fmt, "imm"),
            Self::Mut => write!(fmt, "mut"),
        }
    }
}
impl Display for Scheme {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "forall ")?;
        for var in &self.for_all {
            write!(fmt, "{}, ", var)?;
        }
        write!(fmt, ": {}", &self.ty)?;
        Ok(())
    }
}
