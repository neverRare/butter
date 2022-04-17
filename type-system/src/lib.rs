#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![forbid(unsafe_code)]

use crate::ty::{
    cons::OrderedAnd, Env, Scheme, SchemeMut, Subs, Substitutable, Unifiable, VarState,
};
use hir::{
    expr::{
        Arg, Assign, Binary, BinaryType, Block, Bound, Call, ControlFlow, Element, ElementKind,
        Expr, Field, FieldAccess, Fun, Index, Jump, Literal, PlaceExpr, Range, Record,
        RecordWithSplat, Slice, Tag, Tuple, TupleWithSplat, Unary, UnaryType,
    },
    keyword, pattern,
    statement::{FunDeclare, Statement},
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
        match env.get_ty(Var::new_bare(self.clone())) {
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
        let mut more_subs = Subs::new();
        typed_expr.ty.unify_with(
            Type::Cons(Cons::Record(Keyed {
                fields: once((name.clone(), Type::Var(var.clone()))).collect(),
                rest: Some(var_state.new_var()),
            })),
            &mut more_subs,
            var_state,
        )?;
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
        let mut more_subs = Subs::new();
        typed_expr.ty.unify_with(
            Type::Cons(Cons::Array(Box::new(Type::Var(var.clone())))),
            &mut more_subs,
            var_state,
        )?;
        let mut ty = Type::Var(var);
        ty.substitute(&more_subs)?;
        subs.compose_with(more_subs)?;
        let typed_index = self.index.partial_infer(subs, var_state, env)?;
        typed_index
            .ty
            .unify_with(Type::Cons(Cons::Num), subs, var_state)?;
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
        let mut more_subs = Subs::new();
        typed_expr.ty.unify_with(
            Type::Cons(Cons::Array(Box::new(Type::Var(var)))),
            &mut more_subs,
            var_state,
        )?;
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
                typed.ty.unify_with(
                    Type::Cons(Cons::Array(Box::new(Type::Var(var)))),
                    subs,
                    var_state,
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
                let mut more_subs = Subs::new();
                typed_expr.ty.unify_with(
                    Type::Cons(Cons::Ref(MutType::Var(mut_var), Box::new(ty.clone()))),
                    &mut more_subs,
                    var_state,
                )?;
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
            let mut arr_subs = Subs::new();
            typed_expr
                .ty
                .unify_with(unify_to, &mut arr_subs, var_state)?;
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
                let mut more_subs = Subs::new();
                typed
                    .ty
                    .unify_with(Type::Cons(Cons::Num), &mut more_subs, var_state)?;
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
        let mut more_subs = Subs::new();
        typed_splat
            .ty
            .unify_with(Type::Var(var.clone()), &mut more_subs, var_state)?;
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
        let mut more_subs = Subs::new();
        splat
            .ty
            .unify_with(Type::Var(var), &mut more_subs, var_state)?;
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
                typed.ty.unify_with(ty.clone(), subs, var_state)?;
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
            let mut left_subs = Subs::new();
            left.ty.unify_with(ty.clone(), &mut left_subs, var_state)?;
            ty.substitute(&left_subs)?;
            subs.compose_with(left_subs)?;
            let mut right_subs = Subs::new();
            right
                .ty
                .unify_with(ty.clone(), &mut right_subs, var_state)?;
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
        left.ty.unify_with(op_type.clone(), subs, var_state)?;
        right.ty.unify_with(op_type, subs, var_state)?;
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
        let param_map: HashMap<_, _> = self
            .param
            .iter()
            .map(|var| {
                (
                    var.ident.clone(),
                    (var_state.new_named(var.ident.clone()), var.clone()),
                )
            })
            .collect();
        let mut env = env.clone();
        env.extend(param_map.iter().map(|(var, (new_var, var_hir))| {
            (
                Var::new_bare(var.clone()),
                SchemeMut {
                    is_mut: var_hir.mutable,
                    scheme: Scheme {
                        for_all: HashSet::new(),
                        ty: Type::Var(new_var.clone()),
                    },
                },
            )
        }));
        let return_var = var_state.new_var();
        env.insert(
            Var::new_bare(keyword!("return")),
            SchemeMut {
                is_mut: false,
                scheme: Scheme {
                    for_all: HashSet::new(),
                    ty: Type::Var(return_var.clone()),
                },
            },
        );
        let mut param_ty = Type::Cons(Cons::RecordTuple(OrderedAnd::NonRow(
            self.param
                .iter()
                .map(|var| {
                    (
                        var.ident.clone(),
                        Type::Var(param_map.get(&var.ident).unwrap().0.clone()),
                    )
                })
                .collect::<Vec<_>>()
                .into(),
        )));
        let mut body_subs = Subs::new();
        let body = self.body.partial_infer(&mut body_subs, var_state, &env)?;
        let mut return_ty = Type::Var(return_var);
        return_ty.substitute(&body_subs)?;
        param_ty.substitute(&body_subs)?;
        let param: Vec<_> = self.param.into();
        let typed_param = param
            .into_iter()
            .map(|var| {
                let mut ty = Type::Var(param_map.get(&var.ident).unwrap().0.clone());
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
        let mut body_ty = body.ty;
        let mut body_return_subs = Subs::new();
        return_ty.unify_with(body_ty.clone(), &mut body_return_subs, var_state)?;
        body_ty.substitute(&body_return_subs)?;
        subs.compose_with(body_return_subs)?;
        Ok(Typed {
            ty: Type::Cons(Cons::Fun(Box::new(param_ty), Box::new(body_ty))),
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
                let mut more_subs = Subs::new();
                typed
                    .ty
                    .unify_with(Type::Var(var), &mut more_subs, var_state)?;
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
        let mut subs3 = Subs::new();
        ty1.unify_with(
            Type::Cons(Cons::Fun(
                Box::new(typed2.ty),
                Box::new(Type::Var(var.clone())),
            )),
            &mut subs3,
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
impl Inferable for Assign<()> {
    type TypedSelf = Assign<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let var = self.place.var();
        if let Some(var) = var {
            match env.get_mut(Var::new_bare(var)) {
                Some(true) => (),
                Some(false) => return Err(TypeError::AssignedImm),
                None => return Err(TypeError::UnboundVar),
            }
        }
        // TODO: unify mutability variables of references to `mut`
        let typed_place = self.place.partial_infer(subs, var_state, env)?;
        let typed_expr = self.expr.partial_infer(subs, var_state, env)?;
        typed_place.ty.unify_with(typed_expr.ty, subs, var_state)?;
        Ok(Typed {
            ty: unit(),
            expr: Assign {
                place: typed_place.expr,
                expr: typed_expr.expr,
            },
        })
    }
}
impl Inferable for Box<[Assign<()>]> {
    type TypedSelf = Box<[Assign<Type>]>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let assigns: Vec<_> = self.into();
        let assigns = assigns
            .into_iter()
            .map(|assign| {
                assign
                    .partial_infer(subs, var_state, env)
                    .map(|typed| typed.expr)
            })
            .collect::<Result<Vec<_>, _>>()?
            .into();
        Ok(Typed {
            ty: unit(),
            expr: assigns,
        })
    }
}
impl Inferable for Jump<()> {
    type TypedSelf = Jump<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let typed = match self {
            Jump::Break(_) => todo!(),
            Jump::Continue => todo!(),
            Jump::Return(expr) => {
                let typed_expr = match expr {
                    Some(expr) => expr.partial_infer(subs, var_state, env)?.map(Some),
                    None => Typed {
                        ty: unit(),
                        expr: None,
                    },
                };
                let mut more_subs = Subs::new();
                typed_expr.ty.unify_with(
                    keyword!("return").partial_infer(subs, var_state, env)?.ty,
                    &mut more_subs,
                    var_state,
                )?;
                subs.compose_with(more_subs)?;
                Jump::Return(typed_expr.expr.map(Box::new))
            }
        };
        let var = var_state.new_var();
        Ok(Typed {
            ty: Type::Var(var),
            expr: typed,
        })
    }
}
fn infer_statement(
    subs: &mut Subs,
    env: &mut Env,
    var_state: &mut VarState,
    statement: Statement<()>,
) -> Result<Statement<Type>, TypeError> {
    let typed = match statement {
        Statement::Declare(_) => todo!(),
        Statement::FunDeclare(fun) => {
            let var = Var::new_bare(fun.ident.clone());
            env.remove(var.clone());
            let mut ty = Type::Cons(Cons::Fun(
                Box::new(Type::Var(var_state.new_var())),
                Box::new(Type::Var(var_state.new_var())),
            ));
            env.insert(
                var.clone(),
                SchemeMut {
                    is_mut: false,
                    scheme: Scheme {
                        for_all: HashSet::new(),
                        ty: ty.clone(),
                    },
                },
            );
            let typed_fun = fun.fun.partial_infer(subs, var_state, env)?;
            let mut more_subs = Subs::new();
            typed_fun
                .ty
                .unify_with(ty.clone(), &mut more_subs, var_state)?;
            ty.substitute(&more_subs)?;
            subs.compose_with(more_subs)?;
            env.insert(
                var,
                SchemeMut {
                    is_mut: false,
                    scheme: env.generalize(ty.clone()),
                },
            );
            Statement::FunDeclare(FunDeclare {
                ident: fun.ident,
                fun: typed_fun.expr,
                ty,
            })
        }
        Statement::Expr(expr) => Statement::Expr(expr.partial_infer(subs, var_state, env)?.expr),
    };
    Ok(typed)
}
impl Inferable for Block<()> {
    type TypedSelf = Block<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let mut typed_statement = Vec::with_capacity(self.statement.len());
        let mut env = env.clone();
        let statement: Vec<_> = self.statement.into();
        let mut more_subs = Subs::new();
        for statement in statement {
            let typed = infer_statement(&mut more_subs, &mut env, var_state, statement)?;
            typed_statement.push(typed);
        }
        let typed_expr = match self.expr {
            Some(expr) => expr.partial_infer(subs, var_state, &mut env)?.map(Some),
            None => Typed {
                ty: unit(),
                expr: None,
            },
        };
        let mut ty = typed_expr.ty;
        ty.substitute(&more_subs)?;
        subs.compose_with(more_subs)?;
        Ok(Typed {
            ty,
            expr: Block {
                statement: typed_statement.into(),
                expr: typed_expr.expr.map(Box::new),
            },
        })
    }
}
impl Inferable for ControlFlow<()> {
    type TypedSelf = ControlFlow<Type>;

    fn partial_infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let typed = match self {
            ControlFlow::Block(block) => block
                .partial_infer(subs, var_state, env)?
                .map(ControlFlow::Block),
            ControlFlow::If(_) => todo!(),
            ControlFlow::For(_) => todo!(),
            ControlFlow::While(_) => todo!(),
            ControlFlow::Loop(_) => todo!(),
            ControlFlow::Match(_) => todo!(),
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
                let mut more_subs = Subs::new();
                splat
                    .ty
                    .unify_with(Type::Var(var), &mut more_subs, var_state)?;
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
            Self::Assign(assigns) => assigns
                .partial_infer(subs, var_state, env)?
                .map(Expr::Assign),
            Self::Jump(jump) => jump.partial_infer(subs, var_state, env)?.map(Expr::Jump),
            Self::ControlFlow(control_flow) => control_flow
                .partial_infer(subs, var_state, env)?
                .map(Expr::ControlFlow),
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
