#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![warn(unsafe_code)]

use crate::ty::Env;
use crate::ty::Subs;
use crate::ty::VarState;
use hir::expr::Expr;
use hir::statement::Statement;
use std::mem::transmute;
use std::mem::MaybeUninit;

mod ty;

pub use crate::ty::cons::Cons;
pub use crate::ty::MutType;
pub use crate::ty::Type;
pub use crate::ty::TypeError;
pub use crate::ty::Var;

#[repr(transparent)]
pub struct NoType<'a>(MaybeUninit<Type<'a>>);
impl<'a> Default for NoType<'a> {
    fn default() -> Self {
        Self(MaybeUninit::uninit())
    }
}
struct TypedExpr<'a> {
    ty: Type<'a>,
    expr: Expr<'a, Type<'a>>,
}
fn infer_literal<'a>(expr: &Expr<'a, NoType<'a>>) -> Option<Type<'a>> {
    let cons = match expr {
        Expr::Void => Cons::Unit,
        Expr::True | Expr::False => Cons::Bool,
        Expr::UInt(_) | Expr::Float(_) => Cons::Num,
        _ => return None,
    };
    Some(Type::Cons(cons))
}
fn infer_expr<'a>(
    expr: Expr<'a, NoType<'a>>,
    var_state: &mut VarState<'a>,
    env: &Env<'a>,
) -> Result<(Subs<'a>, TypedExpr<'a>), TypeError> {
    match expr {
        expr @ (Expr::Void | Expr::True | Expr::False | Expr::UInt(_) | Expr::Float(_)) => Ok((
            Subs::new(),
            TypedExpr {
                ty: infer_literal(&expr).unwrap(),
                expr: unsafe { transmute(expr) },
            },
        )),
        _ => todo!(),
    }
}
pub fn infer<'a>(
    statements: Vec<Statement<'a, NoType<'a>>>,
) -> Result<Vec<Statement<'a, Type<'a>>>, TypeError> {
    todo!()
}
