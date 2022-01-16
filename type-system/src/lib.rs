#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![forbid(unsafe_code)]

use crate::ty::{cons::OrderedAnd, Env, Subs, Substitutable, Unifiable, VarState};
use hir::{
    expr::{
        Bound, Element, ElementKind, Expr, Field, FieldAccess, Index, Literal, PlaceExpr, Range,
        Record, RecordWithSplat, Tag,
    },
    statement::Statement,
};
use std::{collections::HashMap, iter::once};
use string_cache::DefaultAtom;

mod ty;

pub use crate::ty::{
    cons::{Cons, Keyed},
    MutType, Type, TypeError, Var,
};

fn unit() -> Type {
    Type::Cons(Cons::RecordTuple(OrderedAnd::NonRow(vec![].into())))
}
struct Typed<T> {
    ty: Type,
    expr: T,
}
impl<T> Typed<T> {
    fn map<U>(self, mapper: impl FnOnce(T) -> U) -> Typed<U> {
        Typed {
            ty: self.ty,
            expr: mapper(self.expr),
        }
    }
}
trait Inferable {
    type TypedSelf;
    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError>;
}
impl Inferable for Literal {
    type TypedSelf = Literal;

    fn partial_infer(
        self,
        _: &mut Subs,
        _: &mut VarState,
        _: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let cons = match self {
            Literal::True | Literal::False => Cons::Bool,
            Literal::UInt(_) | Literal::Float(_) => Cons::Num,
        };
        Ok(Typed {
            ty: Type::Cons(cons),
            expr: self,
        })
    }
}
impl Inferable for DefaultAtom {
    type TypedSelf = DefaultAtom;

    fn partial_infer(
        self,
        _: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        match env.get(&Var {
            name: self.clone(),
            id: 0,
        }) {
            Some(scheme) => Ok(Typed {
                ty: scheme.instantiate(var_state)?,
                expr: self,
            }),
            None => Err(TypeError::UnboundVar),
        }
    }
}
impl Inferable for FieldAccess<()> {
    type TypedSelf = FieldAccess<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let name = self.name;
        let typed_expr = self.expr.partial_infer(subs, var_state, env)?;
        let var = var_state.new_var();
        subs.compose_with(
            Type::Cons(Cons::Record(Keyed {
                fields: once((name.clone(), Type::Var(var.clone()))).collect(),
                rest: Some(var_state.new_var()),
            }))
            .unify_with(typed_expr.ty, var_state)?,
        )?;
        Ok(Typed {
            ty: Type::Var(var),
            expr: FieldAccess {
                expr: Box::new(typed_expr.expr),
                name,
            },
        })
    }
}
impl Inferable for Index<()> {
    type TypedSelf = Index<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let typed_expr = self.expr.partial_infer(subs, var_state, env)?;
        let var = var_state.new_var();
        subs.compose_with(
            Type::Cons(Cons::Array(Box::new(Type::Var(var.clone()))))
                .unify_with(typed_expr.ty, var_state)?,
        )?;
        let typed_index = self.index.partial_infer(subs, var_state, env)?;
        subs.compose_with(Type::Cons(Cons::Num).unify_with(typed_index.ty, var_state)?)?;
        Ok(Typed {
            ty: Type::Var(var),
            expr: Index {
                expr: Box::new(typed_expr.expr),
                index: Box::new(typed_index.expr),
            },
        })
    }
}
impl Inferable for PlaceExpr<()> {
    type TypedSelf = PlaceExpr<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let typed = match self {
            Self::Var(var) => var.partial_infer(subs, var_state, env)?.map(PlaceExpr::Var),
            Self::FieldAccess(expr) => expr
                .partial_infer(subs, var_state, env)?
                .map(PlaceExpr::FieldAccess),
            Self::Index(index) => index
                .partial_infer(subs, var_state, env)?
                .map(PlaceExpr::Index),
            Self::Slice(_) => todo!(),
            Self::Deref(_) => todo!(),
            Self::Len(_) => todo!(),
        };
        Ok(typed)
    }
}
impl Inferable for Box<[Element<()>]> {
    type TypedSelf = Box<[Element<Type>]>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let mut typed_elements = Vec::new();
        let mut ty_var = Type::Var(var_state.new_var());
        let mut arr_ty = Type::Cons(Cons::Array(Box::new(ty_var.clone())));
        for element in Vec::from(self) {
            let typed_expr = element.expr.partial_infer(subs, var_state, env)?;
            typed_elements.push(Element {
                kind: element.kind,
                expr: typed_expr.expr,
            });
            let unify_to = match element.kind {
                ElementKind::Splat => arr_ty.clone(),
                ElementKind::Element => ty_var.clone(),
            };
            let arr_subs = unify_to.unify_with(typed_expr.ty, var_state)?;
            ty_var.substitute(&arr_subs)?;
            arr_ty.substitute(&arr_subs)?;
            // subs.compose_with(arr_subs)?;
        }
        Ok(Typed {
            ty: arr_ty,
            expr: typed_elements.into(),
        })
    }
}
impl Inferable for Range<()> {
    type TypedSelf = Range<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let left = match self.left {
            Some(bound) => {
                let typed = bound.expr.partial_infer(subs, var_state, env)?;
                let more_subs = Type::Cons(Cons::Num).unify_with(typed.ty, var_state)?;
                subs.compose_with(more_subs)?;
                Some(Bound {
                    kind: bound.kind,
                    expr: Box::new(typed.expr),
                })
            }
            None => None,
        };
        let right = match self.right {
            Some(bound) => {
                let typed = bound.expr.partial_infer(subs, var_state, env)?;
                subs.compose_with(Type::Cons(Cons::Num).unify_with(typed.ty, var_state)?)?;
                Some(Bound {
                    kind: bound.kind,
                    expr: Box::new(typed.expr),
                })
            }
            None => None,
        };
        Ok(Typed {
            ty: Type::Cons(Cons::Array(Box::new(Type::Cons(Cons::Num)))),
            expr: (Range { left, right }),
        })
    }
}
impl Inferable for Tag<()> {
    type TypedSelf = Tag<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let (expr, ty) = match self.expr {
            Some(expr) => {
                let typed = expr.partial_infer(subs, var_state, env)?;
                (Some(typed.expr), typed.ty)
            }
            None => (None, unit()),
        };
        Ok(Typed {
            ty: Type::Cons(Cons::Union(Keyed {
                fields: once((self.tag.clone(), ty)).collect(),
                rest: None,
            })),
            expr: Tag {
                tag: self.tag,
                expr: expr.map(Box::new),
            },
        })
    }
}
fn partial_infer_field_list(
    expr: Box<[Field<()>]>,
    ty: &mut HashMap<DefaultAtom, Type>,
    subs: &mut Subs,
    var_state: &mut VarState,
    env: &Env,
) -> Result<Box<[Field<Type>]>, TypeError> {
    let record: Vec<_> = expr.into();
    let mut typed = Vec::with_capacity(record.len());
    for field in record {
        let expr = field.expr.partial_infer(subs, var_state, env)?;
        ty.insert(field.name.clone(), expr.ty);
        typed.push(Field {
            name: field.name,
            expr: expr.expr,
        });
    }
    Ok(typed.into())
}
impl Inferable for Box<[Field<()>]> {
    type TypedSelf = Box<[Field<Type>]>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let mut fields = HashMap::new();
        let typed = partial_infer_field_list(self, &mut fields, subs, var_state, env)?;
        Ok(Typed {
            ty: Type::Cons(Cons::Record(Keyed { fields, rest: None })),
            expr: typed,
        })
    }
}
impl Inferable for RecordWithSplat<()> {
    type TypedSelf = RecordWithSplat<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let mut fields = HashMap::new();
        let typed_left = partial_infer_field_list(self.left, &mut fields, subs, var_state, env)?;
        let typed_splat = self.splat.partial_infer(subs, var_state, env)?;
        let typed_right = partial_infer_field_list(self.right, &mut fields, subs, var_state, env)?;
        let var = var_state.new_var();
        subs.compose_with(Type::Var(var.clone()).unify_with(typed_splat.ty, var_state)?)?;
        Ok(Typed {
            ty: Type::Cons(Cons::Record(Keyed {
                fields,
                rest: Some(var),
            })),
            expr: RecordWithSplat {
                left: typed_left.into(),
                splat: Box::new(typed_splat.expr),
                right: typed_right.into(),
            },
        })
    }
}
impl Inferable for Record<()> {
    type TypedSelf = Record<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let typed = match self {
            Self::Record(record) => record
                .partial_infer(subs, var_state, env)?
                .map(Record::Record),
            Self::RecordWithSplat(record) => record
                .partial_infer(subs, var_state, env)?
                .map(Record::RecordWithSplat),
        };
        Ok(typed)
    }
}
impl Inferable for Expr<()> {
    type TypedSelf = Expr<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let ty_expr = match self {
            Self::Literal(literal) => literal
                .partial_infer(subs, var_state, env)?
                .map(Expr::Literal),
            Self::Place(place) => place.partial_infer(subs, var_state, env)?.map(Expr::Place),
            Self::Array(elements) => elements
                .partial_infer(subs, var_state, env)?
                .map(Expr::Array),
            Self::ArrayRange(range) => range
                .partial_infer(subs, var_state, env)?
                .map(Expr::ArrayRange),
            Self::Tag(tag) => tag.partial_infer(subs, var_state, env)?.map(Expr::Tag),
            Self::Record(record) => record
                .partial_infer(subs, var_state, env)?
                .map(Expr::Record),
            Self::Unit => Typed {
                ty: unit(),
                expr: Expr::Unit,
            },
            Self::Splat(splat) => {
                let splat = splat.partial_infer(subs, var_state, env)?;
                let var = var_state.new_var();
                subs.compose_with(Type::Var(var.clone()).unify_with(splat.ty, var_state)?)?;
                Typed {
                    ty: Type::Cons(Cons::RecordTuple(OrderedAnd::Row(
                        Vec::new(),
                        var,
                        Vec::new(),
                    ))),
                    expr: Expr::Splat(Box::new(splat.expr)),
                }
            }
            Self::Assign(_) => todo!(),
            Self::Unary(_) => todo!(),
            Self::Binary(_) => todo!(),
            Self::Call(_) => todo!(),
            Self::ControlFlow(_) => todo!(),
            Self::Fun(_) => todo!(),
            Self::Jump(_) => todo!(),
            Self::Tuple(_) => todo!(),
        };
        Ok(ty_expr)
    }
}
pub fn infer(statements: Vec<Statement<()>>) -> Result<Vec<Statement<Type>>, TypeError> {
    todo!()
}
pub fn test_infer(expr: Expr<()>) -> Result<Type, TypeError> {
    let mut subs = Subs::new();
    let typed_expr = expr.partial_infer(&mut subs, &mut VarState::new(), &Env::new())?;
    let mut ty = typed_expr.ty;
    ty.substitute(&subs)?;
    Ok(ty)
}
