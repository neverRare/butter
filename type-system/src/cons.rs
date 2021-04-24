use crate::fmt_in_comma;
use crate::Type;
use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) enum Cons<'a> {
    Num,
    Bool,
    Array(Box<Type<'a>>),
    Record(RecordCons<'a>),
    Fun(FunCons<'a>),
    Tuple(Box<[Type<'a>]>),
    // Nullable(Box<Type<'a>>),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct RecordCons<'a> {
    pub fields: HashMap<&'a str, Type<'a>>,
    pub splat_order: SplatOrder<'a>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) enum SplatOrder<'a> {
    Order(Vec<&'a str>),
    Splat(Option<Box<Type<'a>>>),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct FunCons<'a> {
    pub param: Box<Type<'a>>,
    pub result: Box<Type<'a>>,
}
impl<'a> Display for Cons<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match self {
            Self::Num => write!(fmt, "number"),
            Self::Bool => write!(fmt, "boolean"),
            Self::Array(ty) => write!(fmt, "[{}]", ty),
            Self::Record(record) => record.fmt(fmt),
            Self::Fun(fun) => fun.fmt(fmt),
            Self::Tuple(tuple) => {
                write!(fmt, "(")?;
                fmt_in_comma(fmt, tuple.iter(), Type::fmt)?;
                write!(fmt, ")")?;
                Ok(())
            }
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
        match &self.splat_order {
            SplatOrder::Splat(splat) => {
                fmt_in_comma(fmt, &self.fields, |(name, ty), fmt| {
                    writeln!(fmt, "{} = {}", name, ty)
                })?;
                if let Some(splat) = splat {
                    write!(fmt, ", *{}", splat)?;
                }
            }
            SplatOrder::Order(order) => {
                fmt_in_comma(fmt, order, |name, fmt| {
                    writeln!(fmt, "{} = {}", name, self.fields.get(name).unwrap())
                })?;
            }
        }
        write!(fmt, ")")?;
        Ok(())
    }
}
