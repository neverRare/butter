use crate::{
    Atom, PrettyPrintFunScheme, PrettyPrintType, TraverseType, bracket,
    expr::{Expr, Fun},
    intersperse_with_line, intersperse_with_space,
    pattern::Pattern,
};
use pretty::BoxDoc;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<T: PrettyPrintType> {
    Declare(Declare<T>),
    FunDeclare(FunDeclare<T>),
    Expr(Expr<T>),
}
impl<T: PrettyPrintType> Statement<T> {
    pub fn to_doc(&self) -> BoxDoc {
        match self {
            Statement::Declare(declare) => declare.to_doc(),
            Statement::FunDeclare(fun_declare) => fun_declare.to_doc(),
            Statement::Expr(expr) => expr.to_doc(),
        }
    }
}
impl<T: PrettyPrintType> TraverseType for Statement<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut T, &U) -> Result<(), E>,
        for_scheme: fn(&mut T::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        match self {
            Statement::Declare(declare) => declare.traverse_type(data, for_type, for_scheme)?,
            Statement::FunDeclare(fun) => fun.traverse_type(data, for_type, for_scheme)?,
            Statement::Expr(expr) => expr.traverse_type(data, for_type, for_scheme)?,
        }
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Declare<T: PrettyPrintType> {
    pub pattern: Pattern<T>,
    pub expr: Expr<T>,
}
impl<T: PrettyPrintType> Declare<T> {
    pub fn to_doc(&self) -> BoxDoc {
        intersperse_with_space([self.pattern.to_doc(), BoxDoc::text("="), self.expr.to_doc()])
    }
}
impl<T: PrettyPrintType> TraverseType for Declare<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        self.pattern.traverse_type(data, for_type, for_scheme)?;
        self.expr.traverse_type(data, for_type, for_scheme)?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct FunDeclare<T: PrettyPrintType> {
    pub ident: Atom,
    pub fun: Fun<T>,
    pub ty: T::FunScheme,
}
impl<T: PrettyPrintType> FunDeclare<T> {
    pub fn to_doc(&self) -> BoxDoc {
        let fun = if T::TYPED {
            intersperse_with_space([
                BoxDoc::text(&self.ident as &str),
                self.fun.param.to_doc(),
                BoxDoc::text("->"),
                self.fun.body.ty.to_doc().unwrap(),
                BoxDoc::text("=>"),
                self.fun.body.to_doc(),
            ])
        } else {
            intersperse_with_space([
                BoxDoc::text(&self.ident as &str),
                self.fun.param.to_doc(),
                BoxDoc::text("=>"),
                self.fun.body.to_doc(),
            ])
        };
        let generics = self.ty.to_doc();
        if generics.is_empty() {
            fun
        } else {
            intersperse_with_line([
                BoxDoc::concat([
                    BoxDoc::text(":"),
                    bracket(
                        "(",
                        ")",
                        intersperse_with_line(
                            generics
                                .into_iter()
                                .map(|var| BoxDoc::concat([var, BoxDoc::text(",")]).group()),
                        ),
                    ),
                    BoxDoc::text(":"),
                ])
                .group(),
                fun,
            ])
        }
    }
}

impl<T: PrettyPrintType> TraverseType for FunDeclare<T> {
    type Type = T;

    fn traverse_type<U: Clone, E>(
        &mut self,
        data: &U,
        for_type: fn(&mut Self::Type, &U) -> Result<(), E>,
        for_scheme: fn(&mut <Self::Type as PrettyPrintType>::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut data = data.clone();
        for_scheme(&mut self.ty, &mut data)?;
        self.fun.traverse_type(&data, for_type, for_scheme)?;
        Ok(())
    }
}
