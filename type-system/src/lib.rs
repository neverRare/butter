#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![forbid(unsafe_code)]

use crate::ty::{cons::KeyedOrdered, Env, Subs, VarState};
use hir::{
    expr::{
        Bound, Element, ElementKind, Expr, Field, Literal, PlaceExpr, Range, Record,
        RecordWithSplat, Tag,
    },
    statement::Statement,
};
use std::{collections::HashMap, iter::once};

mod ty;

pub use crate::ty::{
    cons::{Cons, Keyed},
    MutType, Type, TypeError, Var,
};

struct TypedExpr<'a> {
    ty: Type<'a>,
    expr: Expr<'a, Type<'a>>,
}
fn unit<'a>() -> Type<'a> {
    Type::Cons(Cons::RecordTuple(KeyedOrdered::NonRow(vec![].into())))
}
fn infer_literal<'a>(literal: Literal) -> Type<'a> {
    let cons = match literal {
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
        Expr::Place(PlaceExpr::FieldAccess(expr)) => {
            let mut subs = Subs::new();
            let name = expr.name;
            let (more_subs, typed_expr) = infer_expr(*expr.expr, var_state, env)?;
            subs.compose_with(more_subs)?;
            let var = var_state.new_var();
            let more_subs = Type::Cons(Cons::Record(Keyed {
                fields: once((name, Type::Var(var))).collect(),
                rest: Some(var_state.new_var()),
            }))
            .unify_with(typed_expr.ty, var_state)?;
            let mut ty = Type::Var(var);
            ty.substitute(&more_subs)?;
            Ok((
                subs,
                TypedExpr {
                    ty,
                    expr: typed_expr.expr,
                },
            ))
        }
        Expr::Place(PlaceExpr::Index(_)) => todo!(),
        Expr::Place(PlaceExpr::Slice(_)) => todo!(),
        Expr::Place(PlaceExpr::Deref(_)) => todo!(),
        Expr::Place(PlaceExpr::Len(_)) => todo!(),
        Expr::Array(elements) => {
            let mut subs = Subs::new();
            let mut typed_elements = Vec::new();
            let mut ty_var = Type::Var(var_state.new_var());
            let mut arr_ty = Type::Cons(Cons::Array(Box::new(ty_var.clone())));
            for element in Vec::from(elements) {
                let (more_subs, typed_expr) = infer_expr(element.expr, var_state, env)?;
                typed_elements.push(Element {
                    kind: element.kind,
                    expr: typed_expr.expr,
                });
                subs.compose_with(more_subs)?;
                let unify_to = match element.kind {
                    ElementKind::Splat => arr_ty.clone(),
                    ElementKind::Element => ty_var.clone(),
                };
                let arr_subs = unify_to.unify_with(typed_expr.ty, var_state)?;
                ty_var.substitute(&arr_subs)?;
                arr_ty.substitute(&arr_subs)?;
                // subs.compose_with(arr_subs)?;
            }
            Ok((
                subs,
                TypedExpr {
                    ty: arr_ty,
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
        Expr::Tag(tag) => {
            let mut subs = Subs::new();
            let (expr, ty) = match tag.expr {
                Some(expr) => {
                    let (more_subs, typed) = infer_expr(*expr, var_state, env)?;
                    subs.compose_with(more_subs)?;
                    (Some(typed.expr), typed.ty)
                }
                None => (None, unit()),
            };
            Ok((
                subs,
                TypedExpr {
                    ty: Type::Cons(Cons::Union(Keyed {
                        fields: once((tag.tag, ty)).collect(),
                        rest: None,
                    })),
                    expr: Expr::Tag(Tag {
                        tag: tag.tag,
                        expr: expr.map(Box::new),
                    }),
                },
            ))
        }
        Expr::Record(Record::Record(record)) => {
            let record: Vec<_> = record.into();
            let mut subs = Subs::new();
            let mut typed = Vec::with_capacity(record.len());
            let mut fields = HashMap::new();
            for field in record {
                let (more_subs, expr) = infer_expr(field.expr, var_state, env)?;
                subs.compose_with(more_subs)?;
                fields.insert(field.name, expr.ty);
                typed.push(Field {
                    name: field.name,
                    expr: expr.expr,
                });
            }
            Ok((
                subs,
                TypedExpr {
                    ty: Type::Cons(Cons::Record(Keyed { fields, rest: None })),
                    expr: Expr::Record(Record::Record(typed.into())),
                },
            ))
        }
        Expr::Record(Record::RecordWithSplat(record)) => {
            let left: Vec<_> = record.left.into();
            let splat = record.splat;
            let right: Vec<_> = record.right.into();
            let mut subs = Subs::new();
            let mut typed_left = Vec::with_capacity(left.len());
            let mut typed_right = Vec::with_capacity(right.len());
            let mut fields = HashMap::new();
            // TODO: this is mostly copy-pasted
            for field in left {
                let (more_subs, expr) = infer_expr(field.expr, var_state, env)?;
                subs.compose_with(more_subs)?;
                fields.insert(field.name, expr.ty);
                typed_left.push(Field {
                    name: field.name,
                    expr: expr.expr,
                });
            }
            for field in right {
                let (more_subs, expr) = infer_expr(field.expr, var_state, env)?;
                subs.compose_with(more_subs)?;
                fields.insert(field.name, expr.ty);
                typed_right.push(Field {
                    name: field.name,
                    expr: expr.expr,
                });
            }
            let (more_subs, rest) = infer_expr(*splat, var_state, env)?;
            subs.compose_with(more_subs)?;
            let var = var_state.new_var();
            subs.compose_with(Type::Var(var).unify_with(rest.ty, var_state)?)?;
            Ok((
                subs,
                TypedExpr {
                    ty: Type::Cons(Cons::Record(Keyed {
                        fields,
                        rest: Some(var),
                    })),
                    expr: Expr::Record(Record::RecordWithSplat(RecordWithSplat {
                        left: typed_left.into(),
                        splat: Box::new(rest.expr),
                        right: typed_right.into(),
                    })),
                },
            ))
        }
        Expr::Unit => Ok((
            Subs::new(),
            TypedExpr {
                ty: unit(),
                expr: Expr::Unit,
            },
        )),
        Expr::Assign(_) => todo!(),
        Expr::Unary(_) => todo!(),
        Expr::Binary(_) => todo!(),
        Expr::Call(_) => todo!(),
        Expr::ControlFlow(_) => todo!(),
        Expr::Fun(_) => todo!(),
        Expr::Jump(_) => todo!(),
        Expr::Splat(_) => todo!(),
        Expr::Tuple(_) => todo!(),
    }
}
pub fn infer(statements: Vec<Statement<()>>) -> Result<Vec<Statement<Type>>, TypeError> {
    todo!()
}
pub fn test_infer(expr: Expr<()>) -> Result<Type, TypeError> {
    let (subs, typed_expr) = infer_expr(expr, &mut VarState::new(), &Env::new())?;
    let mut ty = typed_expr.ty;
    ty.substitute(&subs)?;
    Ok(ty)
}
