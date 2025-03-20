use super::FreeVars;
use crate::ty::{
    Kind, KindedVar, MutType, Subs, Substitutable, Type, Type1, TypeError, Unifiable, Var, VarState,
};
use hir::{Atom, bracket, intersperse_with_line, intersperse_with_space};
use pretty::BoxDoc;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    mem::{replace, swap},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Cons {
    Num,
    Ref(MutType, Box<Type>),
    Array(Box<Type>),
    Fun(Box<Type>, Box<Type>),
    RecordTuple(OrderedAnd<(Atom, Type)>),
    Record(Keyed),
    Tuple(OrderedAnd<Type>),
    Union(Keyed),
}
impl Cons {
    pub fn to_doc(&self) -> BoxDoc {
        match self {
            Self::Num => BoxDoc::text("Num"),
            Self::Ref(mut_type, ty) => intersperse_with_space([
                BoxDoc::concat([BoxDoc::text("&:"), mut_type.to_doc()]).group(),
                ty.to_doc(),
            ]),
            Self::Array(ty) => bracket("[", "]", ty.to_doc()),
            Self::Fun(param, ret) => {
                intersperse_with_space([param.to_doc(), BoxDoc::text("->"), ret.to_doc()])
            }
            Self::RecordTuple(OrderedAnd::NonRow(rec_tup)) => {
                if rec_tup.is_empty() {
                    BoxDoc::text("()")
                } else {
                    let fields = intersperse_with_line(rec_tup.iter().map(|(name, ty)| {
                        BoxDoc::concat([
                            intersperse_with_space([
                                BoxDoc::text(name as &str),
                                BoxDoc::text("="),
                                ty.to_doc(),
                            ]),
                            BoxDoc::text(","),
                        ])
                        .group()
                    }));
                    BoxDoc::concat([BoxDoc::text("ordered"), bracket("(", ")", fields)]).group()
                }
            }
            Self::RecordTuple(OrderedAnd::Row(left, row, right)) => {
                let row =
                    BoxDoc::concat([BoxDoc::text("*"), row.to_doc(), BoxDoc::text(",")]).group();
                let [left, right] = [left, right].map(|rec_tup| {
                    rec_tup.iter().map(|(name, ty)| {
                        BoxDoc::concat([
                            intersperse_with_space([
                                BoxDoc::text(name as &str),
                                BoxDoc::text(":"),
                                ty.to_doc(),
                            ]),
                            BoxDoc::text(","),
                        ])
                        .group()
                    })
                });
                BoxDoc::concat([
                    BoxDoc::text("ordered"),
                    bracket(
                        "(",
                        ")",
                        intersperse_with_line(left.chain([row]).chain(right)),
                    ),
                ])
                .group()
            }
            Self::Record(rec) => {
                let fields = rec.fields.iter().map(|(name, ty)| {
                    BoxDoc::concat([
                        intersperse_with_space([
                            BoxDoc::text(name as &str),
                            BoxDoc::text(":"),
                            ty.to_doc(),
                        ]),
                        BoxDoc::text(","),
                    ])
                    .group()
                });
                match &rec.rest {
                    Some(row) => {
                        let row =
                            BoxDoc::concat([BoxDoc::text("*"), row.to_doc(), BoxDoc::text(",")])
                                .group();
                        bracket("(", ")", intersperse_with_line(fields.chain([row])))
                    }
                    None if rec.fields.is_empty() => BoxDoc::text("()"),
                    None => bracket("(", ")", intersperse_with_line(fields)),
                }
            }
            Self::Tuple(OrderedAnd::NonRow(tup)) => {
                if tup.is_empty() {
                    BoxDoc::text("()")
                } else {
                    bracket(
                        "(",
                        ")",
                        intersperse_with_line(
                            tup.iter()
                                .map(|ty| BoxDoc::concat([ty.to_doc(), BoxDoc::text(",")]).group()),
                        ),
                    )
                }
            }
            Self::Tuple(OrderedAnd::Row(left, row, right)) => {
                let row =
                    BoxDoc::concat([BoxDoc::text("*"), row.to_doc(), BoxDoc::text(",")]).group();
                let [left, right] = [left, right].map(|tup| {
                    tup.iter()
                        .map(|ty| BoxDoc::concat([ty.to_doc(), BoxDoc::text(",")]).group())
                });
                bracket(
                    "(",
                    ")",
                    intersperse_with_line(left.chain([row]).chain(right)),
                )
            }
            Self::Union(union) => {
                let variants = union.fields.iter().map(|(name, ty)| {
                    intersperse_with_space([
                        BoxDoc::concat([BoxDoc::text("@"), BoxDoc::text(name as &str)]).group(),
                        ty.wrap_when_union(),
                    ])
                });
                match &union.rest {
                    Some(row) => {
                        let row = row.to_doc();
                        BoxDoc::intersperse(
                            variants.chain([row]),
                            BoxDoc::concat([BoxDoc::line(), BoxDoc::text("|"), BoxDoc::space()])
                                .group(),
                        )
                    }
                    None if union.fields.is_empty() => BoxDoc::text("never"),
                    None => BoxDoc::intersperse(
                        variants,
                        BoxDoc::concat([BoxDoc::line(), BoxDoc::text("|"), BoxDoc::space()])
                            .group(),
                    ),
                }
            }
        }
    }
    pub fn wrap_when_union(&self) -> BoxDoc {
        let doc = self.to_doc();
        if matches!(self, Cons::Union(_)) {
            bracket("(", ")", doc)
        } else {
            doc
        }
    }
}
impl FreeVars for Cons {
    fn free_vars(&self) -> HashSet<KindedVar> {
        match self {
            Self::Num => HashSet::new(),
            Self::Ref(mutability, ty) => [mutability.free_vars(), ty.free_vars()]
                .into_iter()
                .flatten()
                .collect(),
            Self::Array(ty) => ty.free_vars(),
            Self::Fun(param, ret) => [param, ret]
                .into_iter()
                .map(AsRef::as_ref)
                .flat_map(Type::free_vars)
                .collect(),
            Self::Record(record) => record.free_vars(),
            Self::Tuple(tuple) => tuple.free_vars(),
            Self::RecordTuple(record_tuple) => record_tuple.free_vars(),
            Self::Union(union) => union.free_vars(),
        }
    }
}
impl Substitutable for Cons {
    fn substitute(&mut self, subs: &Subs) -> Result<(), TypeError> {
        match self {
            Self::Num => (),
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
            Self::RecordTuple(record_tuple) => match record_tuple {
                OrderedAnd::NonRow(record_tuple) => {
                    for (_, ty) in record_tuple.iter_mut() {
                        ty.substitute(subs)?;
                    }
                }
                OrderedAnd::Row(_, rest, _) => match subs.get(rest.clone()) {
                    Some(Type1::Type(Type::Var(_) | Type::Cons(Cons::RecordTuple(_)))) => {
                        record_tuple.substitute(subs, |cons| match cons {
                            Cons::RecordTuple(record_tuple) => Some(record_tuple),
                            _ => unreachable!(),
                        })?;
                    }
                    Some(Type1::Type(Type::Cons(other @ (Cons::Record(_) | Cons::Tuple(_))))) => {
                        let record_tuple = match replace(self, Cons::Num) {
                            Cons::RecordTuple(record_tuple) => record_tuple,
                            _ => unreachable!(),
                        };
                        *self = match other {
                            Cons::Record(_) => Cons::Record(record_tuple.into_keyed()),
                            Cons::Tuple(_) => Cons::Tuple(record_tuple.into_ordered()),
                            _ => unreachable!(),
                        };
                        self.substitute(subs)?;
                    }
                    _ => return Err(TypeError::MismatchCons),
                },
            },
            Self::Union(union) => union.substitute(subs, |cons| match cons {
                Cons::Union(ty) => Some(ty),
                _ => None,
            })?,
        }
        Ok(())
    }
}
impl Unifiable for Cons {
    fn unify_with(
        self,
        other: Self,
        subs: &mut Subs,
        var_state: &mut VarState,
    ) -> Result<(), TypeError> {
        match (self, other) {
            (Self::Num, Self::Num) => (),
            (Self::Ref(mut1, ty1), Self::Ref(mut2, ty2)) => {
                mut1.unify_with(mut2, subs, var_state)?;
                ty1.unify_with(*ty2, subs, var_state)?;
            }
            (Self::Array(ty1), Self::Array(ty2)) => ty1.unify_with(*ty2, subs, var_state)?,
            (Self::Fun(param1, ret1), Self::Fun(param2, ret2)) => {
                param1.unify_with(*param2, subs, var_state)?;
                ret1.unify_with(*ret2, subs, var_state)?;
            }
            (Self::Record(rec1), Self::Record(rec2)) => {
                rec1.unify_with(rec2, subs, var_state, Cons::Record)?
            }
            (Self::Record(rec), Self::RecordTuple(rec_tup))
            | (Self::RecordTuple(rec_tup), Self::Record(rec)) => {
                rec.unify_with(rec_tup.into_keyed(), subs, var_state, Cons::Record)?
            }
            (Self::Tuple(tup1), Self::Tuple(tup2)) => {
                tup1.unify_with(tup2, subs, var_state, Cons::Tuple)?
            }
            (Self::Tuple(tup), Self::RecordTuple(rec_tup))
            | (Self::RecordTuple(rec_tup), Self::Tuple(tup)) => {
                tup.unify_with(rec_tup.into_ordered(), subs, var_state, Cons::Tuple)?
            }
            (Self::RecordTuple(rec_tup1), Self::RecordTuple(rec_tup2)) => {
                rec_tup1.unify_with(rec_tup2, subs, var_state, Cons::RecordTuple)?
            }
            (Self::Union(union1), Self::Union(union2)) => {
                union1.unify_with(union2, subs, var_state, Cons::Union)?
            }
            _ => return Err(TypeError::MismatchCons),
        }
        Ok(())
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Keyed {
    pub fields: HashMap<Atom, Type>,
    pub rest: Option<Var>,
}
impl FreeVars for Keyed {
    fn free_vars(&self) -> HashSet<KindedVar> {
        self.fields
            .values()
            .flat_map(Type::free_vars)
            .chain(self.rest.iter().map(|var| KindedVar {
                kind: Kind::Type,
                var: var.clone(),
            }))
            .collect()
    }
}
impl Keyed {
    fn substitute(
        &mut self,
        subs: &Subs,
        matcher: impl FnOnce(Cons) -> Option<Keyed>,
    ) -> Result<(), TypeError> {
        for ty in self.fields.values_mut() {
            ty.substitute(subs)?;
        }
        if let Some(var) = &self.rest {
            match subs.get(var.clone()) {
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
                            fields.insert(key.clone(), ty.clone());
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
        subs: &mut Subs,
        var_state: &mut VarState,
        mut cons: impl FnMut(Keyed) -> Cons,
    ) -> Result<(), TypeError> {
        let mut map1 = self.fields;
        let mut map2 = other.fields;
        for (_, (ty1, ty2)) in intersection(&mut map1, &mut map2) {
            ty1.unify_with(ty2, subs, var_state)?;
        }
        match (self.rest, map1, other.rest, map2) {
            (Some(rest1), map1, Some(rest2), map2) => {
                let new_var = var_state.new_var();
                subs.insert(
                    rest1,
                    Type1::Type(Type::Cons(cons(Keyed {
                        fields: map2,
                        rest: Some(new_var.clone()),
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
                if !map1.is_empty() {
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
                if !map1.is_empty() || !map2.is_empty() {
                    return Err(TypeError::MismatchArity);
                }
            }
        }
        Ok(())
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OrderedAnd<T> {
    NonRow(Box<[T]>),
    Row(Vec<T>, Var, Vec<T>),
}
impl OrderedAnd<(Atom, Type)> {
    fn into_keyed(self) -> Keyed {
        match self {
            Self::NonRow(record) => Keyed {
                fields: record.into_iter().collect(),
                rest: None,
            },
            Self::Row(left, rest, right) => Keyed {
                fields: left.into_iter().chain(right.into_iter()).collect(),
                rest: Some(rest),
            },
        }
    }
    fn into_ordered(self) -> OrderedAnd<Type> {
        match self {
            Self::NonRow(tuple) => {
                OrderedAnd::NonRow(tuple.into_iter().map(|(_, ty)| ty).collect())
            }
            Self::Row(left, rest, right) => OrderedAnd::Row(
                left.into_iter().map(|(_, ty)| ty).collect(),
                rest,
                right.into_iter().map(|(_, ty)| ty).collect(),
            ),
        }
    }
}
impl<T> FreeVars for OrderedAnd<T>
where
    T: FreeVars,
{
    fn free_vars(&self) -> HashSet<KindedVar> {
        match self {
            Self::NonRow(tuple) => tuple.iter().flat_map(T::free_vars).collect(),
            Self::Row(left, rest, right) => left
                .iter()
                .flat_map(T::free_vars)
                .chain([KindedVar {
                    kind: Kind::Type,
                    var: rest.clone(),
                }])
                .chain(right.iter().flat_map(T::free_vars))
                .collect(),
        }
    }
}
impl<T> OrderedAnd<T> {
    fn substitute(
        &mut self,
        subs: &Subs,
        matcher: impl FnOnce(Cons) -> Option<Self>,
    ) -> Result<(), TypeError>
    where
        T: Substitutable,
    {
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
                match subs.get(rest.clone()) {
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
                    Some(_) => return Err(TypeError::MismatchCons),
                    None => (),
                }
            }
        }
        Ok(())
    }
    pub(super) fn unify_with(
        self,
        other: Self,
        subs: &mut Subs,
        var_state: &mut VarState,
        mut cons: impl FnMut(Self) -> Cons,
    ) -> Result<(), TypeError>
    where
        T: Unifiable,
    {
        match (self, other) {
            (Self::NonRow(tup1), Self::NonRow(tup2)) => {
                if tup1.len() != tup2.len() {
                    return Err(TypeError::MismatchArity);
                }
                for (ty1, ty2) in tup1.into_iter().zip(tup2.into_iter()) {
                    ty1.unify_with(ty2, subs, var_state)?;
                }
            }
            (Self::NonRow(tup), Self::Row(left, rest, right))
            | (Self::Row(left, rest, right), Self::NonRow(tup)) => {
                if left.len() + right.len() > tup.len() {
                    return Err(TypeError::MismatchArity);
                }
                let mut left2: Vec<_> = tup.into();
                let mut rest2 = left2.split_off(left.len());
                let right2 = rest2.split_off(rest2.len() - right.len());
                for (ty1, ty2) in left.into_iter().zip(left2.into_iter()) {
                    ty1.unify_with(ty2, subs, var_state)?;
                }
                for (ty1, ty2) in right.into_iter().zip(right2.into_iter()) {
                    ty1.unify_with(ty2, subs, var_state)?;
                }
                subs.insert(
                    rest,
                    Type1::Type(Type::Cons(cons(Self::NonRow(rest2.into())))),
                )
            }
            (Self::Row(_left1, _rest1, _right1), Self::Row(_left2, _rest2, _right2)) => todo!(),
        }
        Ok(())
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
        .collect::<Box<[_]>>()
        .into_iter()
        .map(|key| {
            let a = a.remove(&key).unwrap();
            let b = b.remove(&key).unwrap();
            (key, (a, b))
        })
        .collect()
}
