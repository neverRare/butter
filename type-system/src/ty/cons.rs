use super::FreeVars;
use crate::ty::{
    Kind, KindedVar, MutType, Subs, Substitutable, Type, Type1, TypeError, Unifiable, Var, VarState,
};
use hir::{
    pretty_print::{bracket, line, postfix, prefix, sequence, PrettyPrint},
    Atom,
};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    iter::once,
    mem::{replace, swap},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Cons {
    Num,
    Bool,
    Ref(MutType, Box<Type>),
    Array(Box<Type>),
    Fun(Box<Type>, Box<Type>),
    RecordTuple(OrderedAnd<(Atom, Type)>),
    Record(Keyed),
    Tuple(OrderedAnd<Type>),
    Union(Keyed),
}
impl Cons {
    pub fn to_pretty_print(&self) -> Box<dyn PrettyPrint> {
        match self {
            Self::Num => Box::new("Num".to_string()),
            Self::Bool => Box::new("Bool".to_string()),
            Self::Ref(mut_type, ty) => Box::new(line([
                Box::new("&:".to_string()),
                Box::new(mut_type.to_string()),
                Box::new(" ".to_string()),
                ty.to_pretty_print(),
            ])),
            Self::Array(ty) => Box::new(bracket("[", "]", ty.to_pretty_print())),
            Self::Fun(param, ret) => Box::new(line([
                Box::new(param.to_pretty_print()),
                Box::new(" -> ".to_string()),
                Box::new(ret.to_pretty_print()),
            ])),
            Self::RecordTuple(OrderedAnd::NonRow(rec_tup)) => {
                if rec_tup.is_empty() {
                    Box::new("()".to_string())
                } else {
                    let mut fields = sequence(rec_tup.iter().map(|(name, ty)| {
                        line([
                            Box::new(format!("{name} = ")),
                            Box::new(ty.to_pretty_print()),
                            Box::new(", ".to_string()),
                        ])
                    }));
                    if rec_tup.len() > 1 {
                        fields.multiline_override = Some(true);
                    }
                    Box::new(prefix("ordered", bracket("(", ")", fields)))
                }
            }
            Self::RecordTuple(OrderedAnd::Row(left, row, right)) => {
                let row = Box::new(format!("*{row}, "));
                let [left, right] = [left, right].map(|rec_tup| {
                    rec_tup
                        .iter()
                        .map(|(name, ty)| {
                            line([
                                Box::new(format!("{name} = ")),
                                Box::new(ty.to_pretty_print()),
                                Box::new(", ".to_string()),
                            ])
                        })
                        .map(Box::new)
                        .map(|item| item as Box<dyn PrettyPrint>)
                });
                let list = left
                    .chain(once(Box::new(row) as Box<dyn PrettyPrint>))
                    .chain(right);
                Box::new(prefix("ordered", bracket("(", ")", sequence(list))))
            }
            Self::Record(rec) => {
                let iter = rec.fields.iter().map(|(name, ty)| {
                    line([
                        Box::new(format!("{name} = ")),
                        Box::new(ty.to_pretty_print()),
                        Box::new(", ".to_string()),
                    ])
                });
                match &rec.rest {
                    Some(row) => {
                        let row = Box::new(format!("*{row}, "));
                        Box::new(bracket(
                            "(",
                            ")",
                            sequence(
                                iter.map(Box::new)
                                    .map(|a| a as Box<dyn PrettyPrint>)
                                    .chain(once(Box::new(row) as Box<dyn PrettyPrint>)),
                            ),
                        ))
                    }
                    None if rec.fields.is_empty() => Box::new("()".to_string()),
                    None => {
                        let mut list = sequence(iter);
                        if rec.fields.len() > 1 {
                            list.multiline_override = Some(true);
                        }
                        Box::new(bracket("(", ")", list))
                    }
                }
            }
            Self::Tuple(OrderedAnd::NonRow(tup)) => {
                if tup.is_empty() {
                    Box::new("()".to_string())
                } else {
                    Box::new(bracket(
                        "(",
                        ")",
                        sequence(tup.iter().map(|ty| postfix(", ", ty.to_pretty_print()))),
                    ))
                }
            }
            Self::Tuple(OrderedAnd::Row(left, row, right)) => {
                let row = Box::new(format!("*{row}, "));
                let [left, right] = [left, right].map(|tup| {
                    tup.iter()
                        .map(|ty| postfix(", ", ty.to_pretty_print()))
                        .map(Box::new)
                        .map(|item| item as Box<dyn PrettyPrint>)
                });
                let list = left
                    .chain(once(Box::new(row) as Box<dyn PrettyPrint>))
                    .chain(right);
                Box::new(bracket("(", ")", sequence(list)))
            }
            Self::Union(union) => {
                let iter = union.fields.iter().map(|(name, ty)| {
                    line([
                        Box::new(format!("@{name} ")),
                        Box::new(ty.to_pretty_print()),
                        Box::new(", ".to_string()),
                    ])
                });
                match &union.rest {
                    Some(row) => {
                        let row = Box::new(format!("*{row}, "));
                        Box::new(bracket(
                            "(",
                            ")",
                            sequence(
                                iter.map(Box::new)
                                    .map(|a| a as Box<dyn PrettyPrint>)
                                    .chain(once(Box::new(row) as Box<dyn PrettyPrint>)),
                            ),
                        ))
                    }
                    None if union.fields.is_empty() => Box::new("(@)".to_string()),
                    None => {
                        let mut list = sequence(iter);
                        if union.fields.len() > 1 {
                            list.multiline_override = Some(true);
                        }
                        Box::new(bracket("(", ")", list))
                    }
                }
            }
        }
    }
}
impl FreeVars for Cons {
    fn free_vars(&self) -> HashSet<KindedVar> {
        match self {
            Self::Num | Self::Bool => HashSet::new(),
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
            (Self::Bool, Self::Bool) | (Self::Num, Self::Num) => (),
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
                if map1.is_empty() {
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
                if map1.is_empty() || map2.is_empty() {
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
    fn into_ordered(self) -> OrderedAnd<Type> {
        match self {
            Self::NonRow(tuple) => {
                let tuple: Vec<_> = tuple.into();
                let tuple: Vec<_> = tuple.into_iter().map(|(_, ty)| ty).collect();
                OrderedAnd::NonRow(tuple.into())
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
                .chain(once(KindedVar {
                    kind: Kind::Type,
                    var: rest.clone(),
                }))
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
                let tup1: Vec<_> = tup1.into();
                let tup2: Vec<_> = tup2.into();
                for (ty1, ty2) in tup1.into_iter().zip(tup2.into_iter()) {
                    ty1.unify_with(ty2, subs, var_state)?;
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
        .collect::<Vec<_>>()
        .into_iter()
        .map(|key| {
            let a = a.remove(&key).unwrap();
            let b = b.remove(&key).unwrap();
            (key, (a, b))
        })
        .collect()
}
