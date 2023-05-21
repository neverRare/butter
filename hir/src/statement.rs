use crate::{
    expr::{Expr, Fun},
    pattern::Pattern,
    pretty_print::{
        bracket, line, multiline_sequence, postfix, sequence, PrettyPrint, PrettyPrintTree,
    },
    Atom, PrettyPrintFunScheme, PrettyPrintType,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<T: PrettyPrintType> {
    Declare(Declare<T>),
    FunDeclare(FunDeclare<T>),
    Expr(Expr<T>),
}
impl<T: PrettyPrintType> Statement<T> {
    pub fn traverse_type<U: Clone, E>(
        &mut self,
        _data: &U,
        _for_type: impl FnMut(&mut T, &U) -> Result<(), E>,
        _for_scheme: impl FnMut(&mut T::FunScheme, &mut U) -> Result<(), E>,
    ) -> Result<(), E> {
        todo!();
    }
}
impl<T: PrettyPrintType> PrettyPrint for Statement<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        match self {
            Statement::Declare(declare) => declare.to_pretty_print(),
            Statement::FunDeclare(fun_declare) => fun_declare.to_pretty_print(),
            Statement::Expr(expr) => expr.to_pretty_print(),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Declare<T: PrettyPrintType> {
    pub pattern: Pattern<T>,
    pub expr: Expr<T>,
}
impl<T: PrettyPrintType> PrettyPrint for Declare<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        line([
            self.pattern.to_pretty_print(),
            Box::new(" = ".to_string()),
            self.expr.to_pretty_print(),
        ])
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct FunDeclare<T: PrettyPrintType> {
    pub ident: Atom,
    pub fun: Fun<T>,
    pub ty: T::FunScheme,
}
impl<T: PrettyPrintType> PrettyPrint for FunDeclare<T> {
    fn to_pretty_print(&self) -> Box<dyn PrettyPrintTree> {
        let fun = if T::TYPED {
            line([
                Box::new(self.ident.to_string()),
                self.fun.param.to_pretty_print(),
                Box::new(" -> ".to_string()),
                self.fun.body.ty.to_pretty_print().unwrap(),
                Box::new(" => ".to_string()),
                self.fun.body.to_pretty_print(),
            ])
        } else {
            line([
                Box::new(self.ident.to_string()),
                self.fun.param.to_pretty_print(),
                Box::new(" => ".to_string()),
                self.fun.body.to_pretty_print(),
            ])
        };
        let generics = self.ty.to_pretty_print_generics();
        if generics.is_empty() {
            fun
        } else {
            let generics: Vec<_> = generics.into();
            multiline_sequence([
                line([
                    Box::new(":".to_string()),
                    bracket(
                        "(",
                        ")",
                        sequence(generics.into_iter().map(|var| postfix(", ", var))),
                    ),
                    Box::new(":".to_string()),
                ]),
                fun,
            ])
        }
    }
}
