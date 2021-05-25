use crate::fmt_intersperse;
use crate::HashSet;
use crate::Kind;
use crate::KindedVar;
use crate::MutType;
use crate::Subs;
use crate::Type;
use crate::Type1;
use crate::TypeError;
use crate::Var;
use crate::VarState;
use std::array::IntoIter as ArrayIntoIter;
use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::hash::Hash;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) enum Cons<'a> {
    Unit,
    Num,
    Bool,
    Ref(MutType<'a>, Box<Type<'a>>),
    Array(Box<Type<'a>>),
    Record(RecordCons<'a>),
    Fun(Box<Type<'a>>, Box<Type<'a>>),
    // Tuple(Box<[Type<'a>]>),
    Union(Union<'a>),
}
impl<'a> Cons<'a> {
    pub fn free_vars(&self) -> HashSet<KindedVar<'a>> {
        match self {
            Self::Unit | Self::Num | Self::Bool => HashSet::new(),
            Self::Ref(mutability, ty) => {
                ArrayIntoIter::new([mutability.free_vars(), ty.free_vars()])
                    .flatten()
                    .collect()
            }
            Self::Array(ty) => ty.free_vars(),
            Self::Fun(param, ret) => ArrayIntoIter::new([param, ret])
                .map(AsRef::as_ref)
                .flat_map(Type::free_vars)
                .collect(),
            // FIXME: the following match arms are mostly copy-pasted
            // needs refactor
            Self::Record(record) => record
                .fields
                .values()
                .flat_map(Type::free_vars)
                .chain(record.rest.iter().map(|var| KindedVar {
                    kind: Kind::Type,
                    var: *var,
                }))
                .collect(),
            Self::Union(union) => union
                .union
                .values()
                .flat_map(Type::free_vars)
                .chain(union.rest.iter().map(|var| KindedVar {
                    kind: Kind::Type,
                    var: *var,
                }))
                .collect(),
        }
    }
    pub fn substitute(&mut self, subs: &Subs<'a>) -> Result<(), TypeError> {
        match self {
            Self::Unit | Self::Num | Self::Bool => (),
            Self::Ref(mutability, ty) => {
                mutability.substitute(subs)?;
                ty.substitute(subs)?;
            }
            Self::Array(ty) => ty.substitute(subs)?,
            Self::Fun(param, ret) => {
                param.substitute(subs)?;
                ret.substitute(subs)?;
            }
            // FIXME: the following match arms are mostly copy-pasted
            // needs refactor
            Self::Record(record) => {
                for (_, ty) in &mut record.fields {
                    ty.substitute(subs)?;
                }
                if let Some(var) = &record.rest {
                    let var = *var;
                    match subs.get(var) {
                        Some(Type1::Type(Type::Var(new_var))) => {
                            record.rest = Some(new_var);
                        }
                        Some(Type1::Type(Type::Cons(Self::Record(new_rest)))) => {
                            let new_fields = &new_rest.fields;
                            record.fields.reserve(new_fields.len());
                            for (key, ty) in new_fields {
                                if record.fields.contains_key(key) {
                                    return Err(TypeError::Overlap);
                                } else {
                                    record.fields.insert(*key, ty.clone());
                                }
                            }
                            record.rest = new_rest.rest;
                        }
                        Some(_) => return Err(TypeError::MismatchCons),
                        None => (),
                    }
                }
            }
            Self::Union(union) => {
                for (_, ty) in &mut union.union {
                    ty.substitute(subs)?;
                }
                if let Some(var) = &union.rest {
                    let var = *var;
                    match subs.get(var) {
                        Some(Type1::Type(Type::Var(new_var))) => {
                            union.rest = Some(new_var);
                        }
                        Some(Type1::Type(Type::Cons(Self::Union(new_rest)))) => {
                            let new_fields = &new_rest.union;
                            union.union.reserve(new_fields.len());
                            for (tag, ty) in new_fields {
                                if union.union.contains_key(tag) {
                                    return Err(TypeError::Overlap);
                                } else {
                                    union.union.insert(*tag, ty.clone());
                                }
                            }
                            union.rest = new_rest.rest;
                        }
                        Some(_) => return Err(TypeError::MismatchCons),
                        None => (),
                    }
                }
            }
        }
        Ok(())
    }
    pub fn unify_with(
        self,
        other: Self,
        var_state: &mut VarState<'a>,
    ) -> Result<Subs<'a>, TypeError> {
        let mut subs = Subs::new();
        match (self, other) {
            (Self::Unit, Self::Unit) | (Self::Bool, Self::Bool) | (Self::Num, Self::Num) => (),
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
            // FIXME: the following match arms are mostly copy-pasted
            // needs refactor
            (Self::Record(rec1), Self::Record(rec2)) => {
                let mut map1 = rec1.fields;
                let mut map2 = rec2.fields;
                for (_, (ty1, ty2)) in intersection(&mut map1, &mut map2) {
                    subs.compose_with(ty1.unify_with(ty2, var_state)?)?;
                }
                match (rec1.rest, map1, rec2.rest, map2) {
                    (Some(rest1), map1, Some(rest2), map2) => {
                        let new_var = var_state.new_var();
                        subs.insert(
                            rest1,
                            Type1::Type(Type::Cons(Cons::Record(RecordCons {
                                fields: map2,
                                rest: Some(new_var),
                            }))),
                        );
                        subs.insert(
                            rest2,
                            Type1::Type(Type::Cons(Cons::Record(RecordCons {
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
                            Type1::Type(Type::Cons(Cons::Record(RecordCons {
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
            }
            (Self::Union(union1), Self::Union(union2)) => {
                let mut map1 = union1.union;
                let mut map2 = union2.union;
                for (_, (ty1, ty2)) in intersection(&mut map1, &mut map2) {
                    subs.compose_with(ty1.unify_with(ty2, var_state)?)?;
                }
                match (union1.rest, map1, union2.rest, map2) {
                    (Some(rest1), map1, Some(rest2), map2) => {
                        let new_var = var_state.new_var();
                        subs.insert(
                            rest1,
                            Type1::Type(Type::Cons(Cons::Union(Union {
                                union: map2,
                                rest: Some(new_var),
                            }))),
                        );
                        subs.insert(
                            rest2,
                            Type1::Type(Type::Cons(Cons::Union(Union {
                                union: map1,
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
                            Type1::Type(Type::Cons(Cons::Union(Union {
                                union: map2,
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
            }
            _ => return Err(TypeError::MismatchCons),
        }
        Ok(subs)
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct RecordCons<'a> {
    pub fields: HashMap<&'a str, Type<'a>>,
    pub rest: Option<Var<'a>>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct Union<'a> {
    pub union: HashMap<&'a str, Type<'a>>,
    pub rest: Option<Var<'a>>,
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
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match self {
            Self::Unit => write!(fmt, "unit"),
            Self::Num => write!(fmt, "number"),
            Self::Bool => write!(fmt, "boolean"),
            Self::Ref(mutability, ty) => write!(fmt, "&{} {}", mutability, ty),
            Self::Array(ty) => write!(fmt, "[{}]", ty),
            Self::Record(record) => record.fmt(fmt),
            Self::Fun(param, ret) => write!(fmt, "{} => {}", param, ret),
            Self::Union(union) => union.fmt(fmt),
        }
    }
}
impl<'a> Display for RecordCons<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "(")?;
        fmt_intersperse(fmt, &self.fields, ", ", |(name, ty), fmt| {
            writeln!(fmt, "{} = {}", name, ty)
        })?;
        if let Some(rest) = &self.rest {
            write!(fmt, ", *{}", rest)?;
        }
        write!(fmt, ")")?;
        Ok(())
    }
}
impl<'a> Display for Union<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt_intersperse(fmt, self.union.iter(), " | ", |(tag, assoc), fmt| {
            write!(fmt, "@{} {}", tag, assoc)
        })?;
        if let Some(rest) = &self.rest {
            write!(fmt, " | {}", rest)?;
        }
        Ok(())
    }
}
