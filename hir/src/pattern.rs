use crate::{
    pretty_print::{
        bracket, line, postfix, prefix, sequence, ArraySequence, PrettyPrint, Sequence,
    },
    Atom, PrettyType,
};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    iter::once,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Pattern<T> {
    pub pattern: PatternKind<T>,
    pub ty: T,
}
impl<T> Pattern<T> {
    pub fn field_name(&self) -> Option<Atom> {
        self.pattern.field_name()
    }
}
impl<T: PrettyType> Pattern<T> {
    pub fn to_pretty_print(&self) -> Box<dyn PrettyPrint> {
        let pattern = self.pattern.to_pretty_print();
        match self.ty.to_pretty_print() {
            Some(ty) => Box::new(line([Box::new(ty), Box::new(" : ".to_string()), pattern])),
            None => pattern,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PatternKind<T> {
    True,
    False,
    UInt(u64),
    Int(i64),
    Ignore,
    Var(Var),
    Record(RecordPattern<T>),
    Tuple(ListPattern<T>),
    Param(Box<[TypedVar<T>]>),
    Array(ListPattern<T>),
    Tag(TaggedPattern<T>),
    Ref(Box<Pattern<T>>),
}
impl<T> PatternKind<T> {
    pub fn field_name(&self) -> Option<Atom> {
        match self {
            Self::Var(var) => Some(var.ident.clone()),
            _ => None,
        }
    }
    pub fn to_pretty_print(&self) -> Box<dyn PrettyPrint>
    where
        T: PrettyType,
    {
        match self {
            Self::True => Box::new("true".to_string()),
            Self::False => Box::new("false".to_string()),
            Self::UInt(uint) => Box::new(uint.to_string()),
            Self::Int(int) => Box::new(int.to_string()),
            Self::Ignore => Box::new("_".to_string()),
            Self::Var(var) => Box::new(var.to_string()),
            Self::Record(record) => {
                let iter = record
                    .fields
                    .iter()
                    .map(|(key, pattern)| {
                        line([Box::new(format!("{key} = ")), pattern.to_pretty_print()])
                    })
                    .chain(
                        record
                            .rest
                            .iter()
                            .map(|pattern| prefix("*", pattern.to_pretty_print())),
                    )
                    .map(|pattern| postfix(", ", pattern));
                Box::new(bracket("(", ")", sequence(iter)))
            }
            Self::Tuple(tuple) => Box::new(bracket("(", ")", tuple.to_pretty_print())),
            Self::Param(param) => {
                let iter = param
                    .iter()
                    .map(TypedVar::to_pretty_print)
                    .map(|var| postfix(", ", var));
                Box::new(bracket("(", ")", sequence(iter)))
            }
            Self::Array(arr) => Box::new(bracket("(", ")", arr.to_pretty_print())),
            Self::Tag(tag) => Box::new(tag.to_pretty_print()),
            Self::Ref(pattern) => {
                Box::new(line([Box::new("&".to_string()), pattern.to_pretty_print()]))
            }
        }
    }
}
impl PatternKind<()> {
    pub fn into_untyped(self) -> Pattern<()> {
        Pattern {
            pattern: self,
            ty: (),
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Var {
    pub ident: Atom,
    pub mutable: bool,
    pub bind_to_ref: bool,
}
impl Var {
    pub fn into_untyped(self) -> TypedVar<()> {
        TypedVar { var: self, ty: () }
    }
}
impl Display for Var {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        let mutable = if self.mutable { "mut " } else { "" };
        let bind_to_ref = if self.bind_to_ref { "ref " } else { "" };
        write!(fmt, "{mutable}{bind_to_ref}{}", self.ident)?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypedVar<T> {
    pub var: Var,
    pub ty: T,
}
impl<T> TypedVar<T> {
    pub fn to_pretty_print(&self) -> Box<dyn PrettyPrint>
    where
        T: PrettyType,
    {
        let mut s = self.var.to_string();
        match self.ty.to_pretty_print() {
            Some(ty) => {
                s.push_str(": ");
                Box::new(line([Box::new(s), Box::new(ty)]))
            }
            None => Box::new(s),
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ListPattern<T> {
    List(Box<[Pattern<T>]>),
    ListWithRest(ListWithRest<T>),
}
impl<T> ListPattern<T> {
    fn to_pretty_print(&self) -> Sequence<Vec<Box<dyn PrettyPrint>>>
    where
        T: PrettyType,
    {
        match self {
            ListPattern::List(list) => {
                let iter = list
                    .iter()
                    .map(Pattern::to_pretty_print)
                    .map(|pattern| postfix(", ", pattern));
                sequence(iter)
            }
            ListPattern::ListWithRest(list) => {
                let iter =
                    list.left
                        .iter()
                        .map(Pattern::to_pretty_print)
                        .chain(once(Box::new(prefix("*", list.rest.to_pretty_print()))
                            as Box<dyn PrettyPrint>))
                        .chain(list.right.iter().map(Pattern::to_pretty_print))
                        .map(|pattern| postfix(", ", pattern));
                sequence(iter)
            }
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ListWithRest<T> {
    pub left: Box<[Pattern<T>]>,
    pub rest: Box<Pattern<T>>,
    pub right: Box<[Pattern<T>]>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RecordPattern<T> {
    pub fields: HashMap<Atom, Pattern<T>>,
    pub rest: Option<Box<Pattern<T>>>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TaggedPattern<T> {
    pub tag: Atom,
    pub pattern: Box<Pattern<T>>,
}
impl<T> TaggedPattern<T> {
    fn to_pretty_print(&self) -> ArraySequence<3>
    where
        T: PrettyType,
    {
        let pattern = self.pattern.to_pretty_print();
        let pattern = if T::TYPED {
            pattern
        } else {
            Box::new(bracket("(", ")", pattern))
        };
        let s = format!("{} ", self.tag);
        line([Box::new("@".to_string()), Box::new(s), pattern])
    }
}
