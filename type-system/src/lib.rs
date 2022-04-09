#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![forbid(unsafe_code)]

use crate::ty::{cons::OrderedAnd, Env, Scheme, Subs, Substitutable, Unifiable, VarState};
use hir::{
    expr::{
        Arg, Binary, BinaryType, Bound, Call, Element, ElementKind, Expr, Field, FieldAccess, Fun,
        Index, Literal, PlaceExpr, Range, Record, RecordWithSplat, Slice, Tag, Tuple,
        TupleWithSplat, Unary, UnaryType,
    },
    pattern,
    statement::Statement,
    Atom,
};
use std::{
    collections::{HashMap, HashSet},
    iter::once,
};

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
impl Inferable for Atom {
    type TypedSelf = Atom;

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
        let more_subs = Type::Cons(Cons::Record(Keyed {
            fields: once((name.clone(), Type::Var(var.clone()))).collect(),
            rest: Some(var_state.new_var()),
        }))
        .unify_with(typed_expr.ty, var_state)?;
        let mut ty = Type::Var(var);
        ty.substitute(&more_subs)?;
        subs.compose_with(more_subs)?;
        Ok(Typed {
            ty,
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
        let more_subs = Type::Cons(Cons::Array(Box::new(Type::Var(var.clone()))))
            .unify_with(typed_expr.ty, var_state)?;
        let mut ty = Type::Var(var);
        ty.substitute(&more_subs)?;
        subs.compose_with(more_subs)?;
        let typed_index = self.index.partial_infer(subs, var_state, env)?;
        subs.compose_with(Type::Cons(Cons::Num).unify_with(typed_index.ty, var_state)?)?;
        Ok(Typed {
            ty,
            expr: Index {
                expr: Box::new(typed_expr.expr),
                index: Box::new(typed_index.expr),
            },
        })
    }
}
impl Inferable for Slice<()> {
    type TypedSelf = Slice<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let typed_expr = self.expr.partial_infer(subs, var_state, env)?;
        let var = var_state.new_var();
        let mut elem_ty = Type::Var(var.clone());
        let more_subs = Type::Cons(Cons::Array(Box::new(Type::Var(var))))
            .unify_with(typed_expr.ty, var_state)?;
        elem_ty.substitute(&more_subs)?;
        subs.compose_with(more_subs)?;
        let typed_range = self.range.partial_infer(subs, var_state, env)?;
        Ok(Typed {
            ty: Type::Cons(Cons::Array(Box::new(elem_ty))),
            expr: Slice {
                expr: Box::new(typed_expr.expr),
                range: typed_range.expr,
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
            Self::Slice(slice) => slice
                .partial_infer(subs, var_state, env)?
                .map(PlaceExpr::Slice),
            Self::Len(expr) => {
                let typed = expr.partial_infer(subs, var_state, env)?;
                let var = var_state.new_var();
                subs.compose_with(
                    typed
                        .ty
                        .unify_with(Type::Cons(Cons::Array(Box::new(Type::Var(var)))), var_state)?,
                )?;
                Typed {
                    ty: Type::Cons(Cons::Num),
                    expr: PlaceExpr::Len(Box::new(typed.expr)),
                }
            }
            Self::Deref(expr) => {
                let typed_expr = expr.partial_infer(subs, var_state, env)?;
                let var = var_state.new_var();
                let mut_var = var_state.new_var();
                let mut ty = Type::Var(var);
                let more_subs = Type::Cons(Cons::Ref(MutType::Var(mut_var), Box::new(ty.clone())))
                    .unify_with(typed_expr.ty, var_state)?;
                ty.substitute(&more_subs)?;
                subs.compose_with(more_subs)?;
                Typed {
                    ty,
                    expr: PlaceExpr::Deref(Box::new(typed_expr.expr)),
                }
            }
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
            subs.compose_with(arr_subs)?;
        }
        Ok(Typed {
            ty: arr_ty,
            expr: typed_elements.into(),
        })
    }
}
impl Inferable for Option<Bound<()>> {
    type TypedSelf = Option<Bound<Type>>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let expr = match self {
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
        Ok(Typed { ty: unit(), expr })
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
        let left = self.left.partial_infer(subs, var_state, env)?.expr;
        let right = self.right.partial_infer(subs, var_state, env)?.expr;
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
                rest: Some(var_state.new_var()),
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
    ty: &mut HashMap<Atom, Type>,
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
        let more_subs = Type::Var(var.clone()).unify_with(typed_splat.ty, var_state)?;
        let mut ty = Type::Cons(Cons::Record(Keyed {
            fields,
            rest: Some(var),
        }));
        ty.substitute(&more_subs)?;
        subs.compose_with(more_subs)?;
        Ok(Typed {
            ty,
            expr: RecordWithSplat {
                left: typed_left,
                splat: Box::new(typed_splat.expr),
                right: typed_right,
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
fn infer_tuple(
    tuple: Box<[Expr<()>]>,
    subs: &mut Subs,
    var_state: &mut VarState,
    env: &Env,
) -> Result<(Vec<Type>, Vec<Expr<Type>>), TypeError> {
    let tuple: Vec<_> = tuple.into();
    let len = tuple.len();
    let tuple = tuple.into_iter().try_fold(
        (Vec::with_capacity(len), Vec::with_capacity(len)),
        |(mut ty, mut typed), expr| {
            let inferred = expr.partial_infer(subs, var_state, env)?;
            ty.push(inferred.ty);
            typed.push(inferred.expr);
            Ok((ty, typed))
        },
    )?;
    Ok(tuple)
}
impl Inferable for Box<[Expr<()>]> {
    type TypedSelf = Box<[Expr<Type>]>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let (ty, expr) = infer_tuple(self, subs, var_state, env)?;
        Ok(Typed {
            ty: Type::Cons(Cons::Tuple(OrderedAnd::NonRow(ty.into()))),
            expr: expr.into(),
        })
    }
}
impl Inferable for TupleWithSplat<()> {
    type TypedSelf = TupleWithSplat<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let (left_type, left_expr) = infer_tuple(self.left, subs, var_state, env)?;
        let splat = self.splat.partial_infer(subs, var_state, env)?;
        let (right_type, right_expr) = infer_tuple(self.right, subs, var_state, env)?;
        let var = var_state.new_var();
        let mut ty = Type::Cons(Cons::Tuple(OrderedAnd::Row(
            left_type,
            var.clone(),
            right_type,
        )));
        let more_subs = Type::Var(var).unify_with(splat.ty, var_state)?;
        ty.substitute(&more_subs)?;
        subs.compose_with(more_subs)?;
        Ok(Typed {
            ty,
            expr: TupleWithSplat {
                left: left_expr.into(),
                splat: Box::new(splat.expr),
                right: right_expr.into(),
            },
        })
    }
}
impl Inferable for Tuple<()> {
    type TypedSelf = Tuple<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let typed = match self {
            Self::Tuple(tuple) => tuple.partial_infer(subs, var_state, env)?.map(Tuple::Tuple),
            Self::TupleWithSplat(tuple) => tuple
                .partial_infer(subs, var_state, env)?
                .map(Tuple::TupleWithSplat),
        };
        Ok(typed)
    }
}
impl Inferable for Unary<()> {
    type TypedSelf = Unary<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let typed = self.expr.partial_infer(subs, var_state, env)?;
        let typed = match self.kind {
            // TODO: implement error when cloning function and mutable reference
            // Or maybe not, constraining Clonables may better be implemented
            // by typeclasses or trait, which we don't have yet
            kind @ (UnaryType::Move | UnaryType::Clone) => typed.map(|expr| Unary {
                kind,
                expr: Box::new(expr),
            }),
            kind @ (UnaryType::Minus | UnaryType::Not) => {
                let ty = match kind {
                    UnaryType::Minus => Type::Cons(Cons::Num),
                    UnaryType::Not => Type::Cons(Cons::Bool),
                    _ => unreachable!(),
                };
                subs.compose_with(typed.ty.unify_with(ty.clone(), var_state)?)?;
                Typed {
                    ty,
                    expr: Unary {
                        kind,
                        expr: Box::new(typed.expr),
                    },
                }
            }
            UnaryType::Ref => {
                let var = var_state.new_var();
                Typed {
                    ty: Type::Cons(Cons::Ref(MutType::Var(var), Box::new(typed.ty))),
                    expr: Unary {
                        kind: UnaryType::Ref,
                        expr: Box::new(typed.expr),
                    },
                }
            }
        };
        Ok(typed)
    }
}
impl Inferable for Binary<()> {
    type TypedSelf = Binary<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let left = self.left.partial_infer(subs, var_state, env)?;
        let right = self.right.partial_infer(subs, var_state, env)?;
        if self.kind == BinaryType::Concatenate {
            let var = var_state.new_var();
            let mut ty = Type::Cons(Cons::Array(Box::new(Type::Var(var))));
            let left_subs = ty.clone().unify_with(left.ty, var_state)?;
            ty.substitute(&left_subs)?;
            subs.compose_with(left_subs)?;
            let right_subs = ty.clone().unify_with(right.ty, var_state)?;
            ty.substitute(&right_subs)?;
            subs.compose_with(right_subs)?;
            return Ok(Typed {
                ty,
                expr: Binary {
                    kind: BinaryType::Concatenate,
                    left: Box::new(left.expr),
                    right: Box::new(right.expr),
                },
            });
        }
        let (op_type, return_type) = match self.kind {
            BinaryType::Concatenate => unreachable!(),
            BinaryType::Add
            | BinaryType::Sub
            | BinaryType::Multiply
            | BinaryType::Div
            | BinaryType::FloorDiv
            | BinaryType::Mod => (Type::Cons(Cons::Num), Type::Cons(Cons::Num)),
            BinaryType::Equal
            | BinaryType::NotEqual
            | BinaryType::Greater
            | BinaryType::GreaterEqual
            | BinaryType::Less
            | BinaryType::LessEqual => (Type::Cons(Cons::Num), Type::Cons(Cons::Bool)),
            BinaryType::And | BinaryType::Or | BinaryType::LazyAnd | BinaryType::LazyOr => {
                (Type::Cons(Cons::Bool), Type::Cons(Cons::Bool))
            }
        };
        subs.compose_with(left.ty.unify_with(op_type.clone(), var_state)?)?;
        subs.compose_with(right.ty.unify_with(op_type, var_state)?)?;
        Ok(Typed {
            ty: return_type,
            expr: Binary {
                kind: self.kind,
                left: Box::new(left.expr),
                right: Box::new(right.expr),
            },
        })
    }
}
impl Inferable for Fun<()> {
    type TypedSelf = Fun<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        // TODO: handle `ref` parameters
        let param_var: Vec<_> = self.param.iter().map(|var| var.ident.clone()).collect();
        let param_map: HashMap<_, _> = param_var
            .iter()
            .map(|var| (var.clone(), var_state.new_named(var)))
            .collect();
        let mut env = env.clone();
        env.extend(param_map.iter().map(|(var, new_var)| {
            (
                Var {
                    name: var.clone(),
                    id: 0,
                },
                Scheme {
                    for_all: HashSet::new(),
                    ty: Type::Var(new_var.clone()),
                },
            )
        }));
        let mut param_ty = Type::Cons(Cons::RecordTuple(OrderedAnd::NonRow(
            param_var
                .into_iter()
                .map(|var| (var.clone(), Type::Var(param_map.get(&var).unwrap().clone())))
                .collect::<Vec<_>>()
                .into(),
        )));
        let mut body_subs = Subs::new();
        let body = self.body.partial_infer(&mut body_subs, var_state, &env)?;
        param_ty.substitute(&body_subs)?;
        let param: Vec<_> = self.param.into();
        let typed_param = param
            .into_iter()
            .map(|var| {
                let mut ty = Type::Var(param_map.get(&var.ident).unwrap().clone());
                ty.substitute(&body_subs)?;
                Ok(pattern::Var {
                    ident: var.ident,
                    mutable: var.mutable,
                    bind_to_ref: var.bind_to_ref,
                    ty,
                })
            })
            .collect::<Result<Vec<_>, TypeError>>()?;
        subs.compose_with(body_subs)?;
        Ok(Typed {
            ty: Type::Cons(Cons::Fun(Box::new(param_ty), Box::new(body.ty))),
            expr: Fun {
                param: typed_param.into(),
                body: Box::new(body.expr),
            },
        })
    }
}
impl Inferable for Arg<()> {
    type TypedSelf = Arg<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let typed = match self {
            Arg::Unit => Typed {
                ty: unit(),
                expr: Arg::Unit,
            },
            Arg::Splat(expr) => {
                let typed = expr.partial_infer(subs, var_state, env)?;
                let var = var_state.new_var();
                let mut ty = Type::Cons(Cons::RecordTuple(OrderedAnd::Row(
                    Vec::new(),
                    var.clone(),
                    Vec::new(),
                )));
                let more_subs = Type::Var(var).unify_with(typed.ty, var_state)?;
                ty.substitute(&more_subs)?;
                subs.compose_with(more_subs)?;
                Typed {
                    ty,
                    expr: Arg::Splat(Box::new(typed.expr)),
                }
            }
            Arg::Record(record) => record.partial_infer(subs, var_state, env)?.map(Arg::Record),
            Arg::Tuple(tuple) => tuple.partial_infer(subs, var_state, env)?.map(Arg::Tuple),
        };
        Ok(typed)
    }
}
impl Inferable for Call<()> {
    type TypedSelf = Call<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let var = var_state.new_var();
        let mut subs1 = Subs::new();
        let typed1 = self.expr.partial_infer(&mut subs1, var_state, env)?;
        let mut env2 = env.clone();
        env2.substitute(&subs1)?;
        let mut subs2 = Subs::new();
        let typed2 = self.arg.partial_infer(&mut subs2, var_state, &env2)?;
        let mut ty1 = typed1.ty;
        ty1.substitute(&subs2)?;
        let subs3 = ty1.unify_with(
            Type::Cons(Cons::Fun(
                Box::new(typed2.ty),
                Box::new(Type::Var(var.clone())),
            )),
            var_state,
        )?;
        let mut ty = Type::Var(var);
        ty.substitute(&subs3)?;
        subs.compose_with(subs3)?;
        subs.compose_with(subs2)?;
        subs.compose_with(subs1)?;
        Ok(Typed {
            ty,
            expr: Call {
                expr: Box::new(typed1.expr),
                arg: typed2.expr,
            },
        })
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
            Self::Tuple(tuple) => tuple.partial_infer(subs, var_state, env)?.map(Expr::Tuple),
            Self::Unit => Typed {
                ty: unit(),
                expr: Expr::Unit,
            },
            Self::Splat(splat) => {
                let splat = splat.partial_infer(subs, var_state, env)?;
                let var = var_state.new_var();
                let mut ty = Type::Cons(Cons::RecordTuple(OrderedAnd::Row(
                    Vec::new(),
                    var.clone(),
                    Vec::new(),
                )));
                let more_subs = Type::Var(var).unify_with(splat.ty, var_state)?;
                ty.substitute(&more_subs)?;
                subs.compose_with(more_subs)?;
                Typed {
                    ty,
                    expr: Expr::Splat(Box::new(splat.expr)),
                }
            }
            Self::Unary(unary) => unary.partial_infer(subs, var_state, env)?.map(Expr::Unary),
            Self::Binary(binary) => binary
                .partial_infer(subs, var_state, env)?
                .map(Expr::Binary),
            Self::Fun(fun) => fun.partial_infer(subs, var_state, env)?.map(Expr::Fun),
            Self::Call(call) => call.partial_infer(subs, var_state, env)?.map(Expr::Call),
            Self::Assign(_) => todo!(),
            Self::ControlFlow(_) => todo!(),
            Self::Jump(_) => todo!(),
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
