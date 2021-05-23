use crate::fmt_intersperse;
use crate::Type;
use crate::Var;
use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) enum Cons<'a> {
    Unit,
    Num,
    Bool,
    Array(Box<Type<'a>>),
    Record(RecordCons<'a>),
    Fun(FunCons<'a>),
    Tuple(Box<[Type<'a>]>),
    Union(Union<'a>),
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
        match (&self.rest, &self.order) {
            (rest, None) => {
                fmt_intersperse(fmt, &self.fields, ", ", |(name, ty), fmt| {
                    writeln!(fmt, "{} = {}", name, ty)
                })?;
                if let Some(rest) = rest {
                    write!(fmt, ", *{}", rest)?;
                }
            }
            (None, Some(order)) => {
                fmt_intersperse(fmt, order.iter(), ", ", |name, fmt| {
                    writeln!(fmt, "{} = {}", name, self.fields.get(name).unwrap())
                })?;
            }
            _ => unimplemented!(),
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
