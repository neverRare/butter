#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![forbid(unsafe_code)]

use crate::ty::Env;
use crate::ty::Subs;
use crate::ty::VarState;
use hir::expr::Expr;
use hir::statement::Statement;

mod ty;

pub use crate::ty::MutType;
pub use crate::ty::Type;
pub use crate::ty::TypeError;
pub use crate::ty::Var;

struct TypedExpr<'a> {
    ty: Type<'a>,
    expr: Expr<'a, Type<'a>>,
}
fn infer_expr<'a>(
    expr: Expr<'a, ()>,
    var_state: &mut VarState<'a>,
    env: &Env<'a>,
) -> Result<(Subs<'a>, TypedExpr<'a>), TypeError> {
    todo!()
}
pub fn infer(statements: Vec<Statement<()>>) -> Result<Vec<Statement<Type>>, TypeError> {
    todo!()
}
