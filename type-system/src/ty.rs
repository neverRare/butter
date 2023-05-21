use crate::ty::cons::Cons;
use hir::{
    keyword,
    pretty_print::{PrettyPrint, PrettyPrintTree},
    Atom, PrettyPrintFunScheme, PrettyPrintType,
};
use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display, Formatter},
    hash::Hash,
    iter::once,
};

pub mod cons;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Var {
    pub name: Atom,
    pub id: u32,
}
impl Var {
    pub(super) fn new_bare(name: Atom) -> Self {
        Self { name, id: 0 }
    }
}
impl Display for Var {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match (self.name.as_ref(), self.id) {
            ("", 0) => write!(fmt, "#")?,
            (var, 0) => write!(fmt, "{var}")?,
            (var, id) => write!(fmt, "{var}#{id}")?,
        }
        Ok(())
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub(super) struct VarState(HashMap<Atom, u32>);
impl VarState {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn new_var(&mut self) -> Var {
        self.new_named(keyword!(""))
    }
    pub fn new_named(&mut self, name: Atom) -> Var {
        let Self(map) = self;
        let state = map.entry(name.clone()).or_insert(1);
        let id = *state;
        *state += 1;
        Var { name, id }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Kind {
    Type,
    MutType,
}
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct KindedVar {
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
    fn unify_with(
        self,
        other: Self,
        subs: &mut Subs,
        var_state: &mut VarState,
    ) -> Result<(), TypeError>;
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    Var(Var),
    Cons(Cons),
}
impl PrettyPrint for Type {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        match self {
            Self::Var(var) => Box::new(var.to_string()),
            Self::Cons(cons) => cons.to_pretty_print(),
        }
    }
}
impl PrettyPrintType for Type {
    const TYPED: bool = true;
    type FunScheme = Scheme;

    fn to_pretty_print(&self) -> Option<Box<dyn PrettyPrintTree>> {
        Some(PrettyPrint::to_pretty_print(self))
    }
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
                if let Some(ty) = subs.get(var.clone()) {
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
impl FreeVars for (Atom, Type) {
    fn free_vars(&self) -> HashSet<KindedVar> {
        let (_, ty) = self;
        ty.free_vars()
    }
}
impl Substitutable for (Atom, Type) {
    fn substitute(&mut self, subs: &Subs) -> Result<(), TypeError> {
        let (_, ty) = self;
        ty.substitute(subs)
    }
}
impl Unifiable for Type {
    fn unify_with(
        self,
        other: Self,
        subs: &mut Subs,
        var_state: &mut VarState,
    ) -> Result<(), TypeError> {
        match (self, other) {
            (Self::Cons(cons1), Self::Cons(cons2)) => cons1.unify_with(cons2, subs, var_state)?,
            (Self::Var(var), ty) | (ty, Self::Var(var)) => {
                if ty == Self::Var(var.clone()) {
                    // do nothing
                } else if ty.free_vars().contains(&KindedVar {
                    kind: Kind::Type,
                    var: var.clone(),
                }) {
                    return Err(TypeError::InfiniteOccurrence);
                } else if var.name == keyword!("") {
                    subs.insert(var, Type1::Type(ty));
                } else if let Type::Var(
                    var1 @ Var {
                        name: keyword!(""), ..
                    },
                ) = ty
                {
                    subs.insert(var1, Type1::Type(Type::Var(var)));
                } else {
                    subs.insert(var, Type1::Type(ty));
                }
            }
        }
        Ok(())
    }
}
impl Unifiable for (Atom, Type) {
    fn unify_with(
        self,
        other: Self,
        subs: &mut Subs,
        var_state: &mut VarState,
    ) -> Result<(), TypeError> {
        let (name1, ty1) = self;
        let (name2, ty2) = other;
        if name1 != name2 {
            return Err(TypeError::MismatchName);
        }
        ty1.unify_with(ty2, subs, var_state)?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MutType {
    Var(Var),
    Imm,
    Mut,
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
            if let Some(ty) = subs.get(var.clone()) {
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
    fn unify_with(self, other: Self, subs: &mut Subs, _: &mut VarState) -> Result<(), TypeError> {
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
                } else if var.name == keyword!("") {
                    subs.insert(var, Type1::MutType(ty));
                } else if let MutType::Var(
                    var1 @ Var {
                        name: keyword!(""), ..
                    },
                ) = ty
                {
                    subs.insert(var1, Type1::MutType(MutType::Var(var)));
                } else {
                    subs.insert(var, Type1::MutType(ty));
                }
            }
            _ => return Err(TypeError::MismatchCons),
        }
        Ok(())
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
pub struct Scheme {
    pub for_all: HashSet<KindedVar>,
    pub ty: Type,
}
impl PrettyPrintFunScheme for Scheme {
    fn to_pretty_print_generics(&self) -> Box<[Box<dyn PrettyPrintTree>]> {
        self.for_all
            .iter()
            .map(|var| Box::new(var.var.to_string()) as Box<dyn PrettyPrintTree>)
            .collect::<Vec<_>>()
            .into()
    }
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
    pub(super) fn instantiate(self, var_state: &mut VarState) -> Result<Type, TypeError> {
        let subs = self
            .for_all
            .into_iter()
            .map(|var| {
                let new_var = var_state.new_named(var.var.name.clone());
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
    fn is_empty(&self) -> bool {
        self.hashmap().is_empty()
    }
    fn get(&self, var: Var) -> Option<Type1> {
        self.hashmap().get(&var).map(Type1::clone)
    }
    fn insert(&mut self, var: Var, ty: Type1) {
        self.hashmap_mut().insert(var, ty);
    }
    pub fn filter_off(&mut self, vars: &HashSet<KindedVar>) {
        for var in vars {
            self.hashmap_mut().remove(&var.var);
        }
    }
    pub fn compose_with(&mut self, other: Self) -> Result<(), TypeError> {
        if self.is_empty() {
            *self = other;
        } else {
            let map = self.hashmap_mut();
            for (_, ty) in map.iter_mut() {
                ty.substitute(&other)?;
            }
            map.extend(other.into_hashmap());
        }
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct SchemeMut {
    pub(super) is_mut: bool,
    pub(super) scheme: Scheme,
}
impl FreeVars for SchemeMut {
    fn free_vars(&self) -> HashSet<KindedVar> {
        self.scheme.free_vars()
    }
}
impl Substitutable for SchemeMut {
    fn substitute(&mut self, subs: &Subs) -> Result<(), TypeError> {
        self.scheme.substitute(subs)
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub(super) struct Env(HashMap<Var, SchemeMut>);
impl Env {
    pub fn new() -> Self {
        Self::default()
    }
    fn hashmap(&self) -> &HashMap<Var, SchemeMut> {
        let Self(map) = self;
        map
    }
    fn hashmap_mut(&mut self) -> &mut HashMap<Var, SchemeMut> {
        let Self(map) = self;
        map
    }
    pub fn get_ty(&self, var: Var) -> Option<Scheme> {
        self.hashmap().get(&var).map(|x| Scheme::clone(&x.scheme))
    }
    pub fn get_mut(&self, var: Var) -> Option<bool> {
        self.hashmap().get(&var).map(|x| x.is_mut)
    }
    pub fn insert(&mut self, var: Var, scheme_mut: SchemeMut) -> Option<SchemeMut> {
        self.hashmap_mut().insert(var, scheme_mut)
    }
    pub fn remove(&mut self, var: Var) {
        self.hashmap_mut().remove(&var);
    }
    pub fn generalize(&self, ty: Type) -> Scheme {
        let env_free_vars = self.free_vars();
        let for_all = ty
            .free_vars()
            .into_iter()
            .filter(|var| !env_free_vars.contains(var))
            .collect();
        Scheme { for_all, ty }
    }
}
impl FreeVars for Env {
    fn free_vars(&self) -> HashSet<KindedVar> {
        self.hashmap()
            .values()
            .flat_map(SchemeMut::free_vars)
            .collect()
    }
}
impl Substitutable for Env {
    fn substitute(&mut self, subs: &Subs) -> Result<(), TypeError> {
        for ty in self.hashmap_mut().values_mut() {
            ty.substitute(subs)?;
        }
        Ok(())
    }
}
impl Extend<(Var, SchemeMut)> for Env {
    fn extend<T: IntoIterator<Item = (Var, SchemeMut)>>(&mut self, iter: T) {
        self.hashmap_mut().extend(iter);
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
    AssignedImm,
}
impl Display for TypeError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        write!(fmt, "TypeError")
    }
}
