use crate::{
    pretty_print::{bracket, singleline_sequence, ArraySequence, PrettyPrint},
    Atom, PrettyType,
};
use std::collections::HashMap;

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
    pub fn pretty_print(&self) -> Box<dyn PrettyPrint> {
        let pattern = self.pattern.pretty_print();
        match self.ty.pretty_print() {
            Some(ty) => Box::new(singleline_sequence([
                Box::new(ty),
                Box::new(": ".to_string()),
                pattern,
            ])),
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
    pub fn pretty_print(&self) -> Box<dyn PrettyPrint>
    where
        T: PrettyType,
    {
        match self {
            Self::True => Box::new("true".to_string()),
            Self::False => Box::new("false".to_string()),
            Self::UInt(uint) => Box::new(uint.to_string()),
            Self::Int(int) => Box::new(int.to_string()),
            Self::Ignore => Box::new("_".to_string()),
            Self::Var(var) => Box::new(var.pretty_print()),
            Self::Record(_) => todo!(),
            Self::Tuple(_) => todo!(),
            Self::Param(_) => todo!(),
            Self::Array(_) => todo!(),
            Self::Tag(tag) => Box::new(tag.pretty_print()),
            Self::Ref(pattern) => Box::new(singleline_sequence([
                Box::new("&".to_string()),
                pattern.pretty_print(),
            ])),
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
    pub fn pretty_print(&self) -> String {
        let mutable = if self.mutable { "mut " } else { "" };
        let bind_to_ref = if self.bind_to_ref { "ref " } else { "" };
        format!("{mutable}{bind_to_ref}{}", self.ident)
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypedVar<T> {
    pub var: Var,
    pub ty: T,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ListPattern<T> {
    List(Box<[Pattern<T>]>),
    ListWithRest(ListWithRest<T>),
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
    fn pretty_print(&self) -> ArraySequence<3>
    where
        T: PrettyType,
    {
        let pattern = self.pattern.pretty_print();
        let pattern = if T::TYPED {
            pattern
        } else {
            Box::new(bracket("(", ")", pattern))
        };
        let s = format!("{} ", self.tag);
        singleline_sequence([Box::new("@".to_string()), Box::new(s), pattern])
    }
}
