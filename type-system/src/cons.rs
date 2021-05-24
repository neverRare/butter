use crate::fmt_intersperse;
use crate::HashSet;
use crate::MutType;
use crate::Subs;
use crate::Type;
use crate::Var;
use std::array::IntoIter as ArrayIntoIter;
use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) enum Cons<'a> {
    Unit,
    Num,
    Bool,
    Ref(MutType<'a>, Box<Type<'a>>),
    Array(Box<Type<'a>>),
    Record(RecordCons<'a>),
    Fun(FunCons<'a>),
    Tuple(Box<[Type<'a>]>),
    Union(Union<'a>),
}
impl<'a> Cons<'a> {
    pub fn free_vars(&self) -> HashSet<Var<'a>> {
        match self {
            Self::Unit | Self::Num | Self::Bool => HashSet::new(),
            Self::Ref(mutability, ty) => mutability
                .free_vars()
                .into_iter()
                .chain(ty.free_vars().into_iter())
                .collect(),
            Self::Array(ty) => ty.free_vars(),
            Self::Fun(fun) => ArrayIntoIter::new([&fun.param, &fun.result])
                .map(AsRef::as_ref)
                .flat_map(Type::free_vars)
                .collect(),
            Self::Record(record) => record
                .fields
                .values()
                .flat_map(Type::free_vars)
                .chain(record.rest.iter().copied())
                .collect(),
            Self::Tuple(tuple) => tuple.iter().flat_map(Type::free_vars).collect(),
            Self::Union(union) => union
                .union
                .values()
                .flat_map(Type::free_vars)
                .chain(union.rest.iter().copied())
                .collect(),
        }
    }
    // NOTE: this contains panics, should they be handled as Result?
    pub fn substitute(&mut self, subs: &Subs<'a>) {
        match self {
            Self::Unit | Self::Num | Self::Bool => (),
            Self::Ref(mutability, ty) => {
                mutability.substitute(&subs.mutability);
                ty.substitute(subs);
            }
            Self::Array(ty) => ty.substitute(subs),
            Self::Fun(fun) => {
                fun.param.substitute(subs);
                fun.result.substitute(subs);
            }
            Self::Record(record) => {
                for (_, ty) in &mut record.fields {
                    ty.substitute(subs);
                }
                if let Some(var) = &record.rest {
                    let var = *var;
                    match subs.ty.get(&var) {
                        Some(Type::Var(new_var)) => {
                            record.rest = Some(*new_var);
                        }
                        Some(Type::Cons(Self::Record(new_rest))) => {
                            let new_fields = &new_rest.fields;
                            record.fields.reserve(new_fields.len());
                            for (key, ty) in new_fields {
                                if record.fields.contains_key(key) {
                                    panic!("overlapping key {}", key);
                                } else {
                                    record.fields.insert(*key, ty.clone());
                                }
                            }
                            record.rest = new_rest.rest;
                        }
                        Some(_) => panic!("substituted record rest var as non-record type"),
                        None => (),
                    }
                }
            }
            Self::Tuple(tuple) => {
                for ty in &mut tuple[..] {
                    ty.substitute(subs);
                }
            }
            Self::Union(union) => {
                for (_, ty) in &mut union.union {
                    ty.substitute(subs);
                }
                if let Some(var) = &union.rest {
                    let var = *var;
                    match subs.ty.get(&var) {
                        Some(Type::Var(new_var)) => {
                            union.rest = Some(*new_var);
                        }
                        Some(Type::Cons(Self::Union(new_rest))) => {
                            let new_fields = &new_rest.union;
                            union.union.reserve(new_fields.len());
                            for (tag, ty) in new_fields {
                                if union.union.contains_key(tag) {
                                    panic!("overlapping tag {}", tag);
                                } else {
                                    union.union.insert(*tag, ty.clone());
                                }
                            }
                            union.rest = new_rest.rest;
                        }
                        Some(_) => panic!("substituted union rest var as non-union type"),
                        None => (),
                    }
                }
            }
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct RecordCons<'a> {
    pub fields: HashMap<&'a str, Type<'a>>,
    pub order: Option<Box<[&'a str]>>,
    pub rest: Option<Var<'a>>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct FunCons<'a> {
    pub param: Box<Type<'a>>,
    pub result: Box<Type<'a>>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct Union<'a> {
    pub union: HashMap<&'a str, Type<'a>>,
    pub rest: Option<Var<'a>>,
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
            Self::Fun(fun) => fun.fmt(fmt),
            Self::Tuple(tuple) => {
                write!(fmt, "(")?;
                fmt_intersperse(fmt, tuple.iter(), ", ", Type::fmt)?;
                write!(fmt, ")")?;
                Ok(())
            }
            Self::Union(union) => union.fmt(fmt),
        }
    }
}
impl<'a> Display for FunCons<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{} => {}", self.param, self.result)
    }
}
impl<'a> Display for RecordCons<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "(")?;
        match &self.order {
            Some(order) => fmt_intersperse(fmt, order.iter(), ", ", |name, fmt| {
                writeln!(fmt, "{} = {}", name, self.fields.get(name).unwrap())
            })?,
            None => fmt_intersperse(fmt, &self.fields, ", ", |(name, ty), fmt| {
                writeln!(fmt, "{} = {}", name, ty)
            })?,
        }
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
