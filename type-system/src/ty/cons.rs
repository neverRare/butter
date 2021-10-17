use crate::ty::{
    fmt_intersperse, Kind, KindedVar, MutType, Subs, Type, Type1, TypeError, Var, VarState,
};
use std::{
    array,
    collections::{HashMap, HashSet, VecDeque},
    fmt::{self, Display, Formatter},
    hash::Hash,
    iter::once,
    mem::{replace, swap},
};

// TODO: add tuple, unit, and splat
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Cons<'a> {
    Num,
    Bool,
    Ref(MutType<'a>, Box<Type<'a>>),
    Array(Box<Type<'a>>),
    Fun(Box<Type<'a>>, Box<Type<'a>>),
    RecordTuple(KeyedOrdered<'a>),
    Record(Keyed<'a>),
    Tuple(Ordered<'a>),
    Union(Keyed<'a>),
}
impl<'a> Cons<'a> {
    pub(super) fn free_vars(&self) -> HashSet<KindedVar<'a>> {
        match self {
            Self::Num | Self::Bool => HashSet::new(),
            Self::Ref(mutability, ty) => {
                array::IntoIter::new([mutability.free_vars(), ty.free_vars()])
                    .flatten()
                    .collect()
            }
            Self::Array(ty) => ty.free_vars(),
            Self::Fun(param, ret) => array::IntoIter::new([param, ret])
                .map(AsRef::as_ref)
                .flat_map(Type::free_vars)
                .collect(),
            Self::Record(record) => record.free_vars(),
            Self::Tuple(tuple) => tuple.free_vars(),
            Self::RecordTuple(record_tuple) => record_tuple.free_vars(),
            Self::Union(union) => union.free_vars(),
        }
    }
    pub(super) fn substitute(&mut self, subs: &Subs<'a>) -> Result<(), TypeError> {
        match self {
            Self::Num | Self::Bool => (),
            Self::Ref(mutability, ty) => {
                mutability.substitute(subs)?;
                ty.substitute(subs)?;
            }
            Self::Array(ty) => ty.substitute(subs)?,
            Self::Fun(param, ret) => {
                param.substitute(subs)?;
                ret.substitute(subs)?;
            }
            Self::Record(record) => record.substitute(subs, |cons| match cons {
                Cons::Record(ty) => Some(ty),
                Cons::RecordTuple(ty) => Some(ty.into_keyed()),
                _ => None,
            })?,
            Self::Tuple(tuple) => tuple.substitute(subs, |cons| match cons {
                Cons::Tuple(ty) => Some(ty),
                Cons::RecordTuple(ty) => Some(ty.into_ordered()),
                _ => None,
            })?,
            Self::RecordTuple(record_tuple) => todo!(),
            Self::Union(union) => union.substitute(subs, |cons| match cons {
                Cons::Union(ty) => Some(ty),
                _ => None,
            })?,
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
            (Self::Bool, Self::Bool) | (Self::Num, Self::Num) => (),
            (Self::Ref(mut1, ty1), Self::Ref(mut2, ty2)) => {
                subs.compose_with(mut1.unify_with(mut2)?)?;
                subs.compose_with(ty1.unify_with(*ty2, var_state)?)?;
            }
            (Self::Array(ty1), Self::Array(ty2)) => {
                subs.compose_with(ty1.unify_with(*ty2, var_state)?)?
            }
            (Self::Fun(param1, ret1), Self::Fun(param2, ret2)) => {
                subs.compose_with(param1.unify_with(*param2, var_state)?)?;
                subs.compose_with(ret1.unify_with(*ret2, var_state)?)?;
            }
            (Self::Record(rec1), Self::Record(rec2)) => {
                subs.compose_with(rec1.unify_with(rec2, var_state, |ty| Cons::Record(ty))?)?
            }
            (Self::Record(rec), Self::RecordTuple(rec_tup))
            | (Self::RecordTuple(rec_tup), Self::Record(rec)) => subs.compose_with(
                rec.unify_with(rec_tup.into_keyed(), var_state, |ty| Cons::Record(ty))?,
            )?,
            (Self::Tuple(tup1), Self::Tuple(tup2)) => {
                subs.compose_with(tup1.unify_with(tup2, var_state, |ty| Cons::Tuple(ty))?)?
            }
            (Self::Tuple(tup), Self::RecordTuple(rec_tup))
            | (Self::RecordTuple(rec_tup), Self::Tuple(tup)) => subs.compose_with(
                tup.unify_with(rec_tup.into_ordered(), var_state, |ty| Cons::Tuple(ty))?,
            )?,
            (Self::RecordTuple(rec_tup1), Self::RecordTuple(rec_tup2)) => todo!(),
            (Self::Union(union1), Self::Union(union2)) => {
                subs.compose_with(union1.unify_with(union2, var_state, |ty| Cons::Union(ty))?)?
            }
            _ => return Err(TypeError::MismatchCons),
        }
        Ok(subs)
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Keyed<'a> {
    pub fields: HashMap<&'a str, Type<'a>>,
    pub rest: Option<Var<'a>>,
}
impl<'a> Keyed<'a> {
    fn free_vars(&self) -> HashSet<KindedVar<'a>> {
        self.fields
            .values()
            .flat_map(Type::free_vars)
            .chain(self.rest.iter().map(|var| KindedVar {
                kind: Kind::Type,
                var: *var,
            }))
            .collect()
    }
    fn substitute(
        &mut self,
        subs: &Subs<'a>,
        matcher: impl FnOnce(Cons) -> Option<Keyed>,
    ) -> Result<(), TypeError> {
        for (_, ty) in &mut self.fields {
            ty.substitute(subs)?;
        }
        if let Some(var) = &self.rest {
            let var = *var;
            match subs.get(var) {
                Some(Type1::Type(Type::Var(new_var))) => {
                    self.rest = Some(new_var);
                }
                Some(Type1::Type(Type::Cons(cons))) => {
                    let new_rest = matcher(cons).ok_or(TypeError::MismatchCons)?;
                    let new_fields = &new_rest.fields;
                    let fields = &mut self.fields;
                    fields.reserve(new_fields.len());
                    for (key, ty) in new_fields {
                        if fields.contains_key(key) {
                            return Err(TypeError::Overlap);
                        } else {
                            fields.insert(key, ty.clone());
                        }
                    }
                    // NOTE: why there's no HashMap::reserve_exact??
                    if new_rest.rest.is_none() {
                        fields.shrink_to_fit();
                    }
                    self.rest = new_rest.rest;
                }
                Some(_) => return Err(TypeError::MismatchCons),
                None => (),
            }
        }
        Ok(())
    }
    pub(super) fn unify_with(
        self,
        other: Self,
        var_state: &mut VarState<'a>,
        mut cons: impl FnMut(Keyed) -> Cons,
    ) -> Result<Subs<'a>, TypeError> {
        let mut subs = Subs::new();
        let mut map1 = self.fields;
        let mut map2 = other.fields;
        for (_, (ty1, ty2)) in intersection(&mut map1, &mut map2) {
            subs.compose_with(ty1.unify_with(ty2, var_state)?)?;
        }
        match (self.rest, map1, other.rest, map2) {
            (Some(rest1), map1, Some(rest2), map2) => {
                let new_var = var_state.new_var();
                subs.insert(
                    rest1,
                    Type1::Type(Type::Cons(cons(Keyed {
                        fields: map2,
                        rest: Some(new_var),
                    }))),
                );
                subs.insert(
                    rest2,
                    Type1::Type(Type::Cons(cons(Keyed {
                        fields: map1,
                        rest: Some(new_var),
                    }))),
                );
            }
            (Some(rest1), map1, None, map2) | (None, map2, Some(rest1), map1) => {
                if map1.len() != 0 {
                    return Err(TypeError::MismatchArity);
                }
                subs.insert(
                    rest1,
                    Type1::Type(Type::Cons(cons(Keyed {
                        fields: map2,
                        rest: None,
                    }))),
                );
            }
            (None, map1, None, map2) => {
                if map1.len() != 0 || map2.len() != 0 {
                    return Err(TypeError::MismatchArity);
                }
            }
        }
        Ok(subs)
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Ordered<'a> {
    NonRow(Box<[Type<'a>]>),
    Row(Vec<Type<'a>>, Var<'a>, Vec<Type<'a>>),
}
impl<'a> Ordered<'a> {
    fn free_vars(&self) -> HashSet<KindedVar<'a>> {
        match self {
            Self::NonRow(tuple) => tuple.iter().flat_map(Type::free_vars).collect(),
            Self::Row(left, rest, right) => left
                .iter()
                .flat_map(Type::free_vars)
                .chain(once(KindedVar {
                    kind: Kind::Type,
                    var: *rest,
                }))
                .chain(right.iter().flat_map(Type::free_vars))
                .collect(),
        }
    }
    fn substitute(
        &mut self,
        subs: &Subs<'a>,
        matcher: impl FnOnce(Cons) -> Option<Ordered>,
    ) -> Result<(), TypeError> {
        match self {
            Self::NonRow(tuple) => {
                for ty in tuple.iter_mut() {
                    ty.substitute(subs)?
                }
            }
            Self::Row(left, rest, right) => {
                for ty in left.iter_mut() {
                    ty.substitute(subs)?
                }
                for ty in right.iter_mut() {
                    ty.substitute(subs)?
                }
                match subs.get(*rest) {
                    Some(Type1::Type(Type::Var(var))) => {
                        *rest = var;
                    }
                    Some(Type1::Type(Type::Cons(cons))) => {
                        match matcher(cons).ok_or(TypeError::MismatchCons)? {
                            Self::Row(more_left, new_rest, mut more_right) => {
                                left.extend(more_left);
                                *rest = new_rest;
                                let mut temp = vec![];
                                swap(right, &mut temp);
                                more_right.extend(temp);
                                *right = more_right;
                            }
                            Self::NonRow(new_tuple) => {
                                let new_tuple: Vec<_> = new_tuple.into();
                                let (left, right) = {
                                    match replace(self, Self::NonRow(vec![].into())) {
                                        Self::Row(left, _, right) => (left, right),
                                        _ => unreachable!(),
                                    }
                                };
                                *self = Self::NonRow(
                                    left.into_iter()
                                        .chain(new_tuple.into_iter())
                                        .chain(right.into_iter())
                                        .collect(),
                                );
                            }
                        }
                    }
                    _ => return Err(TypeError::MismatchCons),
                    None => (),
                }
            }
        }
        Ok(())
    }
    pub(super) fn unify_with(
        self,
        other: Self,
        var_state: &mut VarState<'a>,
        mut cons: impl FnMut(Ordered) -> Cons,
    ) -> Result<Subs<'a>, TypeError> {
        let mut subs = Subs::new();
        match (self, other) {
            (Self::NonRow(tup1), Self::NonRow(tup2)) => {
                if tup1.len() != tup2.len() {
                    return Err(TypeError::MismatchArity);
                }
                let tup1: Vec<_> = tup1.into();
                let tup2: Vec<_> = tup2.into();
                for (ty1, ty2) in tup1.into_iter().zip(tup2.into_iter()) {
                    subs.compose_with(ty1.unify_with(ty2, var_state)?)?;
                }
            }
            (Self::NonRow(tup), Self::Row(left, rest, right))
            | (Self::Row(left, rest, right), Self::NonRow(tup)) => {
                let tup: Vec<_> = tup.into();
                if left.len() + right.len() > tup.len() {
                    return Err(TypeError::MismatchArity);
                }
                let mut left2 = tup;
                let mut rest2 = left2.split_off(left.len());
                let right2 = rest2.split_off(rest2.len() - right.len());
                for (ty1, ty2) in left.into_iter().zip(left2.into_iter()) {
                    subs.compose_with(ty1.unify_with(ty2, var_state)?)?;
                }
                for (ty1, ty2) in right.into_iter().zip(right2.into_iter()) {
                    subs.compose_with(ty1.unify_with(ty2, var_state)?)?;
                }
                subs.insert(
                    rest,
                    Type1::Type(Type::Cons(cons(Self::NonRow(rest2.into())))),
                )
            }
            (Self::Row(left1, rest1, right1), Self::Row(left2, rest2, right2)) => todo!(),
        }
        Ok(subs)
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum KeyedOrdered<'a> {
    NonRow(Box<[(&'a str, Type<'a>)]>),
    Row(Vec<(&'a str, Type<'a>)>, Var<'a>, Vec<(&'a str, Type<'a>)>),
}
impl<'a> KeyedOrdered<'a> {
    fn into_keyed(self) -> Keyed<'a> {
        match self {
            Self::NonRow(record) => {
                let record: Vec<_> = record.into();
                Keyed {
                    fields: record.into_iter().collect(),
                    rest: None,
                }
            }
            Self::Row(left, rest, right) => Keyed {
                fields: left.into_iter().chain(right.into_iter()).collect(),
                rest: Some(rest),
            },
        }
    }
    fn into_ordered(self) -> Ordered<'a> {
        match self {
            Self::NonRow(tuple) => {
                let tuple: Vec<_> = tuple.into();
                let tuple: Vec<_> = tuple.into_iter().map(|(_, ty)| ty).collect();
                Ordered::NonRow(tuple.into())
            }
            Self::Row(left, rest, right) => Ordered::Row(
                left.into_iter().map(|(_, ty)| ty).collect(),
                rest,
                right.into_iter().map(|(_, ty)| ty).collect(),
            ),
        }
    }
    fn free_vars(&self) -> HashSet<KindedVar<'a>> {
        match self {
            Self::NonRow(record) => record.iter().flat_map(|(_, ty)| ty.free_vars()).collect(),
            Self::Row(left, rest, right) => left
                .iter()
                .flat_map(|(_, ty)| ty.free_vars())
                .chain(once(KindedVar {
                    kind: Kind::Type,
                    var: *rest,
                }))
                .chain(right.iter().flat_map(|(_, ty)| ty.free_vars()))
                .collect(),
        }
    }
}
fn intersection<K, A, B>(a: &mut HashMap<K, A>, b: &mut HashMap<K, B>) -> HashMap<K, (A, B)>
where
    K: Hash + Eq + Clone,
{
    // TODO: avoid allocation, return an iterator instead of `HashMap`, and
    // remove the `Clone` requirement for `K`
    a.keys()
        .filter(|key| b.contains_key(key))
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .map(|key| {
            let a = a.remove(&key).unwrap();
            let b = b.remove(&key).unwrap();
            (key, (a, b))
        })
        .collect()
}
impl<'a> Display for Cons<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Self::Num => write!(fmt, "number"),
            Self::Bool => write!(fmt, "boolean"),
            Self::Ref(mutability, ty) => write!(fmt, "&{} {}", mutability, ty),
            Self::Array(ty) => write!(fmt, "[{}]", ty),
            Self::Record(record) => {
                write!(fmt, "(")?;
                fmt_intersperse(fmt, &record.fields, ", ", |(name, ty), fmt| {
                    write!(fmt, "{} = {}", name, ty)
                })?;
                if let Some(rest) = &record.rest {
                    write!(fmt, ", *{}", rest)?;
                }
                write!(fmt, ")")?;
                Ok(())
            }
            Self::Tuple(tuple) => todo!(),
            Self::RecordTuple(record_tuple) => todo!(),
            Self::Fun(param, ret) => write!(fmt, "{} => {}", param, ret),
            Self::Union(union) => {
                fmt_intersperse(fmt, union.fields.iter(), " | ", |(tag, assoc), fmt| {
                    write!(fmt, "@{} {}", tag, assoc)
                })?;
                if let Some(rest) = &union.rest {
                    write!(fmt, " | {}", rest)?;
                }
                Ok(())
            }
        }
    }
}
