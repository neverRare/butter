#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![forbid(unsafe_code)]

use crate::ty::Env;
use crate::ty::Subs;
use crate::ty::VarState;
use hir::expr::Bound;
use hir::expr::Element;
use hir::expr::ElementKind;
use hir::expr::Expr;
use hir::expr::Literal;
use hir::expr::PlaceExpr;
use hir::expr::Range;
use hir::statement::Statement;

mod ty;

pub use crate::ty::cons::Cons;
pub use crate::ty::MutType;
pub use crate::ty::Type;
pub use crate::ty::TypeError;
pub use crate::ty::Var;

struct TypedExpr<'a> {
    ty: Type<'a>,
    expr: Expr<'a, Type<'a>>,
}
fn infer_literal<'a>(literal: Literal) -> Type<'a> {
    let cons = match literal {
        Literal::Void => Cons::Unit,
        Literal::True | Literal::False => Cons::Bool,
        Literal::UInt(_) | Literal::Float(_) => Cons::Num,
    };
    Type::Cons(cons)
}
fn infer_expr<'a>(
    expr: Expr<'a, ()>,
    var_state: &mut VarState<'a>,
    env: &Env<'a>,
) -> Result<(Subs<'a>, TypedExpr<'a>), TypeError> {
    match expr {
        Expr::Literal(literal) => Ok((
            Subs::new(),
            TypedExpr {
                ty: infer_literal(literal),
                expr: Expr::Literal(literal),
            },
        )),
        Expr::Place(PlaceExpr::Var(var)) => match env.get(Var { name: var, id: 0 }) {
            Some(scheme) => Ok((
                Subs::new(),
                TypedExpr {
                    ty: scheme.instantiate(var_state)?,
                    expr: Expr::Place(PlaceExpr::Var(var)),
                },
            )),
            None => Err(TypeError::UnboundVar),
        },
        Expr::Array(elements) => {
            let mut subs = Subs::new();
            let mut ty = Type::Cons(Cons::Array(Box::new(Type::Var(var_state.new_var()))));
            let mut typed_elements = Vec::new();
            for element in Vec::from(elements) {
                let (more_subs, typed_expr) = infer_expr(element.expr, var_state, env)?;
                typed_elements.push(Element {
                    kind: element.kind,
                    expr: typed_expr.expr,
                });
                subs.compose_with(more_subs)?;
                let inferred_ty = match element.kind {
                    ElementKind::Splat => typed_expr.ty,
                    ElementKind::Element => Type::Cons(Cons::Array(Box::new(typed_expr.ty))),
                };
                let more_subs = ty.clone().unify_with(inferred_ty, var_state)?;
                subs.compose_with(more_subs)?;
                ty.substitute(&subs)?;
            }
            Ok((
                subs,
                TypedExpr {
                    ty,
                    expr: Expr::Array(typed_elements.into()),
                },
            ))
        }
        Expr::ArrayRange(range) => {
            let mut subs = Subs::new();
            let left = match range.left {
                Some(bound) => {
                    let (more_subs, typed) = infer_expr(*bound.expr, var_state, env)?;
                    subs.compose_with(more_subs)?;
                    let more_subs = Type::Cons(Cons::Num).unify_with(typed.ty, var_state)?;
                    subs.compose_with(more_subs)?;
                    Some(Bound {
                        kind: bound.kind,
                        expr: Box::new(typed.expr),
                    })
                }
                None => None,
            };
            let right = match range.right {
                Some(bound) => {
                    let (more_subs, typed) = infer_expr(*bound.expr, var_state, env)?;
                    subs.compose_with(more_subs)?;
                    let more_subs = Type::Cons(Cons::Num).unify_with(typed.ty, var_state)?;
                    subs.compose_with(more_subs)?;
                    Some(Bound {
                        kind: bound.kind,
                        expr: Box::new(typed.expr),
                    })
                }
                None => None,
            };
            Ok((
                subs,
                TypedExpr {
                    ty: Type::Cons(Cons::Array(Box::new(Type::Cons(Cons::Num)))),
                    expr: Expr::ArrayRange(Range { left, right }),
                },
            ))
        }
        _ => todo!(),
    }
}
pub fn infer(statements: Vec<Statement<()>>) -> Result<Vec<Statement<Type>>, TypeError> {
    todo!()
}
