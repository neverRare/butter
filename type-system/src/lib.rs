#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![forbid(unsafe_code)]

use expr::Inferable;
use hir::{expr::Expr, statement::Statement, TraverseType};
use ty::{Env, Subs, Substitutable, VarState};

mod expr;
mod pattern;
mod ty;

pub use crate::ty::{
    cons::{Cons, Keyed},
    MutType, Type, TypeError, Var,
};
struct Typed<T> {
    ty: Type,
    value: T,
}
impl<T> Typed<T> {
    fn map<U>(self, mapper: impl FnOnce(T) -> U) -> Typed<U> {
        Typed {
            ty: self.ty,
            value: mapper(self.value),
        }
    }
}
fn substitute_hir(hir: &mut impl TraverseType<Type = Type>, subs: &Subs) -> Result<(), TypeError> {
    hir.traverse_type(
        subs,
        |ty, subs| ty.substitute(subs),
        |scheme, subs| {
            scheme.substitute(subs)?;
            subs.filter_off(&scheme.for_all);
            Ok(())
        },
    )
}
pub fn infer(_statements: Vec<Statement<()>>) -> Result<Vec<Statement<Type>>, TypeError> {
    todo!()
}
pub fn test_infer(expr: Expr<()>) -> Result<Type, TypeError> {
    let mut subs = Subs::new();
    let typed_expr = expr.infer(&mut subs, &mut VarState::new(), &Env::new())?;
    let mut ty = typed_expr.ty;
    ty.substitute(&subs)?;
    Ok(ty)
}
