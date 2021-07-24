use hir::statement::Statement;

mod ty;

pub use crate::ty::MutType;
pub use crate::ty::Type;
pub use crate::ty::TypeError;
pub use crate::ty::Var;

pub fn infer(statements: Vec<Statement<()>>) -> Result<Vec<Statement<Type>>, TypeError> {
    todo!()
}
