use crate::{
    pattern::InferablePattern,
    substitute_hir,
    ty::{
        cons::OrderedAnd,
        cons::{Cons, Keyed},
        Env, MutType, Subs, Substitutable, Type, TypeError, Unifiable, Var, VarState,
    },
    Typed,
};
use hir::{
    expr::{
        Arg, Assign, Binary, BinaryType, Block, Bound, Call, Collection, ControlFlow, Element,
        ElementKind, Expr, ExprKind, Field, FieldAccess, If, Index, Jump, Literal, PlaceExpr,
        Range, Slice, Tag, Unary, UnaryType, WithSplat,
    },
    keyword,
    statement::{Declare, Statement},
    Atom,
};
use std::{collections::HashMap, iter::once};

pub(super) fn unit() -> Type {
    Type::Cons(Cons::RecordTuple(OrderedAnd::NonRow(vec![].into())))
}
pub(super) trait Inferable {
    type TypedSelf;
    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError>
    where
        Self: Sized,
    {
        self.infer_with_mut(subs, var_state, env)
            .map(|(_, typed)| typed)
    }
    fn infer_with_mut(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<(Option<Var>, Typed<Self::TypedSelf>), TypeError>
    where
        Self: Sized,
    {
        self.infer(subs, var_state, env).map(|typed| (None, typed))
    }
}
impl Inferable for Literal {
    type TypedSelf = Literal;

    fn infer(
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
            value: self,
        })
    }
}
impl Inferable for Atom {
    type TypedSelf = Atom;

    fn infer(
        self,
        _: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        match env.get_ty(Var::new_bare(self.clone())) {
            Some(scheme) => Ok(Typed {
                ty: scheme.instantiate(var_state)?,
                value: self,
            }),
            None => Err(TypeError::UnboundVar),
        }
    }
}
impl Inferable for FieldAccess<()> {
    type TypedSelf = FieldAccess<Type>;

    fn infer_with_mut(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<(Option<Var>, Typed<Self::TypedSelf>), TypeError> {
        let name = self.name;
        let (mut_var, typed_expr) = self.expr.infer_with_mut(subs, var_state, env)?;
        let operand_ty = typed_expr.ty;
        let mut operand_expr = typed_expr.value;
        let var = var_state.new_var();
        let mut operand_subs = Subs::new();
        operand_ty.unify_with(
            Type::Cons(Cons::Record(Keyed {
                fields: once((name.clone(), Type::Var(var.clone()))).collect(),
                rest: Some(var_state.new_var()),
            })),
            &mut operand_subs,
            var_state,
        )?;
        let mut ty = Type::Var(var);
        ty.substitute(&operand_subs)?;
        substitute_hir(&mut operand_expr, &operand_subs)?;
        subs.compose_with(operand_subs)?;
        Ok((
            mut_var,
            Typed {
                ty,
                value: FieldAccess {
                    expr: Box::new(operand_expr),
                    name,
                },
            },
        ))
    }
}
impl Inferable for Index<()> {
    type TypedSelf = Index<Type>;

    fn infer_with_mut(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<(Option<Var>, Typed<Self::TypedSelf>), TypeError> {
        let (mut_var, typed_expr) = self.expr.infer_with_mut(subs, var_state, env)?;
        let operand_ty = typed_expr.ty;
        let mut operand_expr = typed_expr.value;
        let var = var_state.new_var();
        let mut operand_subs = Subs::new();
        operand_ty.unify_with(
            Type::Cons(Cons::Array(Box::new(Type::Var(var.clone())))),
            &mut operand_subs,
            var_state,
        )?;
        let mut ty = Type::Var(var);
        ty.substitute(&operand_subs)?;
        substitute_hir(&mut operand_expr, &operand_subs)?;
        subs.compose_with(operand_subs)?;
        let typed_index = self.index.infer(subs, var_state, env)?;
        let index_ty = typed_index.ty;
        let mut index_expr = typed_index.value;
        let mut index_subs = Subs::new();
        index_ty.unify_with(Type::Cons(Cons::Num), &mut index_subs, var_state)?;
        substitute_hir(&mut index_expr, &index_subs)?;
        subs.compose_with(index_subs)?;
        Ok((
            mut_var,
            Typed {
                ty,
                value: Index {
                    expr: Box::new(operand_expr),
                    index: Box::new(index_expr),
                },
            },
        ))
    }
}
impl Inferable for Slice<()> {
    type TypedSelf = Slice<Type>;

    fn infer_with_mut(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<(Option<Var>, Typed<Self::TypedSelf>), TypeError> {
        let (mut_var, typed_expr) = self.expr.infer_with_mut(subs, var_state, env)?;
        let operand_ty = typed_expr.ty;
        let mut operand_expr = typed_expr.value;
        let var = var_state.new_var();
        let mut elem_ty = Type::Var(var.clone());
        let mut operand_subs = Subs::new();
        operand_ty.unify_with(
            Type::Cons(Cons::Array(Box::new(Type::Var(var)))),
            &mut operand_subs,
            var_state,
        )?;
        elem_ty.substitute(&operand_subs)?;
        substitute_hir(&mut operand_expr, &operand_subs)?;
        subs.compose_with(operand_subs)?;
        let typed_range = self.range.infer(subs, var_state, env)?;
        Ok((
            mut_var,
            Typed {
                ty: Type::Cons(Cons::Array(Box::new(elem_ty))),
                value: Slice {
                    expr: Box::new(operand_expr),
                    range: typed_range.value,
                },
            },
        ))
    }
}
impl Inferable for PlaceExpr<()> {
    type TypedSelf = PlaceExpr<Type>;

    fn infer_with_mut(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<(Option<Var>, Typed<PlaceExpr<Type>>), TypeError> {
        let mut_typed = match self {
            Self::Var(var) => (None, var.infer(subs, var_state, env)?.map(PlaceExpr::Var)),
            Self::FieldAccess(expr) => {
                let (mut_var, typed) = expr.infer_with_mut(subs, var_state, env)?;
                (mut_var, typed.map(PlaceExpr::FieldAccess))
            }
            Self::Index(index) => {
                let (mut_var, typed) = index.infer_with_mut(subs, var_state, env)?;
                (mut_var, typed.map(PlaceExpr::Index))
            }
            Self::Slice(slice) => {
                let (mut_var, typed) = slice.infer_with_mut(subs, var_state, env)?;
                (mut_var, typed.map(PlaceExpr::Slice))
            }
            Self::Len(expr) => {
                let (mut_var, typed) = expr.infer_with_mut(subs, var_state, env)?;
                let operand_ty = typed.ty;
                let mut operand_expr = typed.value;
                let var = var_state.new_var();
                let mut operand_subs = Subs::new();
                operand_ty.unify_with(
                    Type::Cons(Cons::Array(Box::new(Type::Var(var)))),
                    &mut operand_subs,
                    var_state,
                )?;
                substitute_hir(&mut operand_expr, &operand_subs)?;
                subs.compose_with(operand_subs)?;
                (
                    mut_var,
                    Typed {
                        ty: Type::Cons(Cons::Num),
                        value: PlaceExpr::Len(Box::new(operand_expr)),
                    },
                )
            }
            Self::Deref(expr) => {
                let (mut_var, typed_expr) = expr.infer_with_mut(subs, var_state, env)?;
                let operand_ty = typed_expr.ty;
                let mut operand_expr = typed_expr.value;
                let var = var_state.new_var();
                let mut_var = mut_var.unwrap_or_else(|| var_state.new_var());
                let mut ty = Type::Var(var);
                let mut operand_subs = Subs::new();
                operand_ty.unify_with(
                    Type::Cons(Cons::Ref(
                        MutType::Var(mut_var.clone()),
                        Box::new(ty.clone()),
                    )),
                    &mut operand_subs,
                    var_state,
                )?;
                ty.substitute(&operand_subs)?;
                substitute_hir(&mut operand_expr, &operand_subs)?;
                subs.compose_with(operand_subs)?;
                (
                    Some(mut_var),
                    Typed {
                        ty,
                        value: PlaceExpr::Deref(Box::new(operand_expr)),
                    },
                )
            }
        };
        Ok(mut_typed)
    }
}
impl Inferable for Box<[Element<()>]> {
    type TypedSelf = Box<[Element<Type>]>;

    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let mut typed_elements = Vec::new();
        let mut ty_var = Type::Var(var_state.new_var());
        let mut arr_ty = Type::Cons(Cons::Array(Box::new(ty_var.clone())));
        for element in Vec::from(self) {
            let typed_expr = element.expr.infer(subs, var_state, env)?;
            let elem_ty = typed_expr.ty;
            let mut elem_expr = typed_expr.value;
            let unify_to = match element.kind {
                ElementKind::Splat => arr_ty.clone(),
                ElementKind::Element => ty_var.clone(),
            };
            let mut elem_subs = Subs::new();
            elem_ty.unify_with(unify_to, &mut elem_subs, var_state)?;
            ty_var.substitute(&elem_subs)?;
            arr_ty.substitute(&elem_subs)?;
            substitute_hir(&mut elem_expr, &elem_subs)?;
            typed_elements.push(Element {
                kind: element.kind,
                expr: elem_expr,
            });
            subs.compose_with(elem_subs)?;
        }
        Ok(Typed {
            ty: arr_ty,
            value: typed_elements.into(),
        })
    }
}
impl Inferable for Option<Bound<()>> {
    type TypedSelf = Option<Bound<Type>>;

    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let expr = match self {
            Some(bound) => {
                let typed = bound.expr.infer(subs, var_state, env)?;
                let bound_ty = typed.ty;
                let mut bound_expr = typed.value;
                let mut bound_subs = Subs::new();
                bound_ty.unify_with(Type::Cons(Cons::Num), &mut bound_subs, var_state)?;
                substitute_hir(&mut bound_expr, &bound_subs)?;
                subs.compose_with(bound_subs)?;
                Some(Bound {
                    kind: bound.kind,
                    expr: Box::new(bound_expr),
                })
            }
            None => None,
        };
        Ok(Typed {
            ty: unit(),
            value: expr,
        })
    }
}
impl Inferable for Range<()> {
    type TypedSelf = Range<Type>;

    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let left = self.left.infer(subs, var_state, env)?.value;
        let right = self.right.infer(subs, var_state, env)?.value;
        Ok(Typed {
            ty: Type::Cons(Cons::Array(Box::new(Type::Cons(Cons::Num)))),
            value: (Range { left, right }),
        })
    }
}
impl Inferable for Tag<()> {
    type TypedSelf = Tag<Type>;

    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let (expr, ty) = match self.expr {
            Some(expr) => {
                let typed = expr.infer(subs, var_state, env)?;
                (Some(typed.value), typed.ty)
            }
            None => (None, unit()),
        };
        Ok(Typed {
            ty: Type::Cons(Cons::Union(Keyed {
                fields: once((self.tag.clone(), ty)).collect(),
                rest: Some(var_state.new_var()),
            })),
            value: Tag {
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
        let expr = field.expr.infer(subs, var_state, env)?;
        ty.insert(field.name.clone(), expr.ty);
        typed.push(Field {
            name: field.name,
            expr: expr.value,
        });
    }
    Ok(typed.into())
}
impl Inferable for Box<[Field<()>]> {
    type TypedSelf = Box<[Field<Type>]>;

    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let mut fields = HashMap::new();
        let typed = partial_infer_field_list(self, &mut fields, subs, var_state, env)?;
        Ok(Typed {
            ty: Type::Cons(Cons::Record(Keyed { fields, rest: None })),
            value: typed,
        })
    }
}
impl Inferable for WithSplat<Field<()>, ()> {
    type TypedSelf = WithSplat<Field<Type>, Type>;

    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let mut fields = HashMap::new();
        let typed_left = partial_infer_field_list(self.left, &mut fields, subs, var_state, env)?;
        let typed_splat = self.splat.infer(subs, var_state, env)?;
        let splat_ty = typed_splat.ty;
        let mut splat_expr = typed_splat.value;
        let typed_right = partial_infer_field_list(self.right, &mut fields, subs, var_state, env)?;
        let var = var_state.new_var();
        let mut splat_subs = Subs::new();
        splat_ty.unify_with(Type::Var(var.clone()), &mut splat_subs, var_state)?;
        let mut ty = Type::Cons(Cons::Record(Keyed {
            fields,
            rest: Some(var),
        }));
        ty.substitute(&splat_subs)?;
        substitute_hir(&mut splat_expr, &splat_subs)?;
        subs.compose_with(splat_subs)?;
        Ok(Typed {
            ty,
            value: WithSplat {
                left: typed_left,
                splat: Box::new(splat_expr),
                right: typed_right,
            },
        })
    }
}
impl Inferable for Collection<Field<()>, ()> {
    type TypedSelf = Collection<Field<Type>, Type>;

    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let typed = match self {
            Self::Collection(record) => record
                .infer(subs, var_state, env)?
                .map(Collection::Collection),
            Self::WithSplat(record) => record
                .infer(subs, var_state, env)?
                .map(Collection::WithSplat),
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
            let inferred = expr.infer(subs, var_state, env)?;
            ty.push(inferred.ty);
            typed.push(inferred.value);
            Ok((ty, typed))
        },
    )?;
    Ok(tuple)
}
impl Inferable for Box<[Expr<()>]> {
    type TypedSelf = Box<[Expr<Type>]>;

    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let (ty, expr) = infer_tuple(self, subs, var_state, env)?;
        Ok(Typed {
            ty: Type::Cons(Cons::Tuple(OrderedAnd::NonRow(ty.into()))),
            value: expr.into(),
        })
    }
}
impl Inferable for WithSplat<Expr<()>, ()> {
    type TypedSelf = WithSplat<Expr<Type>, Type>;

    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let (left_type, left_expr) = infer_tuple(self.left, subs, var_state, env)?;
        let splat = self.splat.infer(subs, var_state, env)?;
        let splat_ty = splat.ty;
        let mut splat_expr = splat.value;
        let (right_type, right_expr) = infer_tuple(self.right, subs, var_state, env)?;
        let var = var_state.new_var();
        let mut ty = Type::Cons(Cons::Tuple(OrderedAnd::Row(
            left_type,
            var.clone(),
            right_type,
        )));
        let mut splat_subs = Subs::new();
        splat_ty.unify_with(Type::Var(var), &mut splat_subs, var_state)?;
        ty.substitute(&splat_subs)?;
        substitute_hir(&mut splat_expr, &splat_subs)?;
        subs.compose_with(splat_subs)?;
        Ok(Typed {
            ty,
            value: WithSplat {
                left: left_expr.into(),
                splat: Box::new(splat_expr),
                right: right_expr.into(),
            },
        })
    }
}
impl Inferable for Collection<Expr<()>, ()> {
    type TypedSelf = Collection<Expr<Type>, Type>;

    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let typed = match self {
            Self::Collection(tuple) => tuple
                .infer(subs, var_state, env)?
                .map(Collection::Collection),
            Self::WithSplat(tuple) => tuple
                .infer(subs, var_state, env)?
                .map(Collection::WithSplat),
        };
        Ok(typed)
    }
}
impl Inferable for Unary<()> {
    type TypedSelf = Unary<Type>;

    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let (mut_var, typed) = self.expr.infer_with_mut(subs, var_state, env)?;
        let typed = match self.kind {
            // TODO: implement error when cloning function and mutable reference
            // Or maybe not, constraining Clonables may better be implemented
            // by typeclasses or trait, which we don't have yet
            kind @ (UnaryType::Move | UnaryType::Clone) => typed.map(|expr| Unary {
                kind,
                expr: Box::new(expr),
            }),
            kind @ (UnaryType::Minus | UnaryType::Not) => {
                let operand_ty = typed.ty;
                let mut operand_expr = typed.value;
                let ty = match kind {
                    UnaryType::Minus => Type::Cons(Cons::Num),
                    UnaryType::Not => Type::Cons(Cons::Bool),
                    _ => unreachable!(),
                };
                let mut operand_subs = Subs::new();
                operand_ty.unify_with(ty.clone(), &mut operand_subs, var_state)?;
                substitute_hir(&mut operand_expr, &operand_subs)?;
                subs.compose_with(operand_subs)?;
                Typed {
                    ty,
                    value: Unary {
                        kind,
                        expr: Box::new(operand_expr),
                    },
                }
            }
            UnaryType::Ref => {
                let operand_ty = typed.ty;
                let operand_expr = typed.value;
                let var = mut_var.unwrap_or_else(|| var_state.new_var());
                Typed {
                    ty: Type::Cons(Cons::Ref(MutType::Var(var), Box::new(operand_ty))),
                    value: Unary {
                        kind: UnaryType::Ref,
                        expr: Box::new(operand_expr),
                    },
                }
            }
        };
        Ok(typed)
    }
}
impl Inferable for Binary<()> {
    type TypedSelf = Binary<Type>;

    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let left = self.left.infer(subs, var_state, env)?;
        let left_ty = left.ty;
        let mut left_expr = left.value;
        let right = self.right.infer(subs, var_state, env)?;
        let right_ty = right.ty;
        let mut right_expr = right.value;
        let (op_type, mut return_type) = match self.kind {
            BinaryType::Concatenate => {
                let var = var_state.new_var();
                let ty = Type::Cons(Cons::Array(Box::new(Type::Var(var))));
                (ty.clone(), ty)
            }
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
        let mut left_subs = Subs::new();
        left_ty.unify_with(op_type.clone(), &mut left_subs, var_state)?;
        return_type.substitute(&left_subs)?;
        substitute_hir(&mut left_expr, &left_subs)?;
        subs.compose_with(left_subs)?;
        let mut right_subs = Subs::new();
        right_ty.unify_with(op_type, &mut right_subs, var_state)?;
        return_type.substitute(&right_subs)?;
        substitute_hir(&mut right_expr, &right_subs)?;
        subs.compose_with(right_subs)?;
        Ok(Typed {
            ty: return_type,
            value: Binary {
                kind: self.kind,
                left: Box::new(left_expr),
                right: Box::new(right_expr),
            },
        })
    }
}
// impl Inferable for Fun<()> {
//     type TypedSelf = Fun<Type>;

//     fn infer(
//         self,
//         subs: &mut Subs,
//         var_state: &mut VarState,
//         env: &Env,
//     ) -> Result<Typed<Self::TypedSelf>, TypeError> {
//         // TODO: handle `ref` parameters
//         let param_map: HashMap<_, _> = self
//             .param
//             .iter()
//             .map(|var| {
//                 (
//                     var.ident.clone(),
//                     (var_state.new_named(var.ident.clone()), var.clone()),
//                 )
//             })
//             .collect();
//         let mut env = env.clone();
//         env.extend(param_map.iter().map(|(var, (new_var, var_hir))| {
//             (
//                 Var::new_bare(var.clone()),
//                 SchemeMut {
//                     is_mut: var_hir.mutable,
//                     scheme: Scheme {
//                         for_all: HashSet::new(),
//                         ty: Type::Var(new_var.clone()),
//                     },
//                 },
//             )
//         }));
//         let return_var = var_state.new_var();
//         env.insert(
//             Var::new_bare(keyword!("return")),
//             SchemeMut {
//                 is_mut: false,
//                 scheme: Scheme {
//                     for_all: HashSet::new(),
//                     ty: Type::Var(return_var.clone()),
//                 },
//             },
//         );
//         let mut param_ty = Type::Cons(Cons::RecordTuple(OrderedAnd::NonRow(
//             self.param
//                 .iter()
//                 .map(|var| {
//                     (
//                         var.ident.clone(),
//                         Type::Var(param_map.get(&var.ident).unwrap().0.clone()),
//                     )
//                 })
//                 .collect::<Vec<_>>()
//                 .into(),
//         )));
//         let mut body_subs = Subs::new();
//         let body = self.body.infer(&mut body_subs, var_state, &env)?;
//         let mut return_ty = Type::Var(return_var);
//         return_ty.substitute(&body_subs)?;
//         param_ty.substitute(&body_subs)?;
//         let param: Vec<_> = self.param.into();
//         let typed_param = param
//             .into_iter()
//             .map(|var| {
//                 let mut ty = Type::Var(param_map.get(&var.ident).unwrap().0.clone());
//                 ty.substitute(&body_subs)?;
//                 Ok(pattern::Var {
//                     ident: var.ident,
//                     mutable: var.mutable,
//                     bind_to_ref: var.bind_to_ref,
//                     ty,
//                 })
//             })
//             .collect::<Result<Vec<_>, TypeError>>()?;
//         subs.compose_with(body_subs)?;
//         let mut body_ty = body.ty;
//         let mut body_return_subs = Subs::new();
//         return_ty.unify_with(body_ty.clone(), &mut body_return_subs, var_state)?;
//         body_ty.substitute(&body_return_subs)?;
//         subs.compose_with(body_return_subs)?;
//         Ok(Typed {
//             ty: Type::Cons(Cons::Fun(Box::new(param_ty), Box::new(body_ty))),
//             value: Fun {
//                 param: typed_param.into(),
//                 body: Box::new(body.value),
//             },
//         })
//     }
// }
impl Inferable for Arg<()> {
    type TypedSelf = Arg<Type>;

    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let typed = match self {
            Arg::Unit => Typed {
                ty: unit(),
                value: Arg::Unit,
            },
            Arg::Splat(expr) => {
                let typed = expr.infer(subs, var_state, env)?;
                let operand_ty = typed.ty;
                let mut operand_expr = typed.value;
                let var = var_state.new_var();
                let mut ty = Type::Cons(Cons::RecordTuple(OrderedAnd::Row(
                    Vec::new(),
                    var.clone(),
                    Vec::new(),
                )));
                let mut operand_subs = Subs::new();
                operand_ty.unify_with(Type::Var(var), &mut operand_subs, var_state)?;
                ty.substitute(&operand_subs)?;
                substitute_hir(&mut operand_expr, &operand_subs)?;
                subs.compose_with(operand_subs)?;
                Typed {
                    ty,
                    value: Arg::Splat(Box::new(operand_expr)),
                }
            }
            Arg::Record(record) => record.infer(subs, var_state, env)?.map(Arg::Record),
            Arg::Tuple(tuple) => tuple.infer(subs, var_state, env)?.map(Arg::Tuple),
        };
        Ok(typed)
    }
}
impl Inferable for Call<()> {
    type TypedSelf = Call<Type>;

    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let var = var_state.new_var();
        let mut subs1 = Subs::new();
        let typed1 = self.expr.infer(&mut subs1, var_state, env)?;
        let mut env2 = env.clone();
        env2.substitute(&subs1)?;
        let mut subs2 = Subs::new();
        let typed2 = self.arg.infer(&mut subs2, var_state, &env2)?;
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
        let mut callee_expr = typed1.value;
        let mut arg_expr = typed2.value;
        substitute_hir(&mut callee_expr, &subs3)?;
        substitute_hir(&mut arg_expr, &subs3)?;
        subs.compose_with(subs3)?;
        subs.compose_with(subs2)?;
        subs.compose_with(subs1)?;
        Ok(Typed {
            ty,
            value: Call {
                expr: Box::new(callee_expr),
                arg: arg_expr,
            },
        })
    }
}
impl Inferable for Assign<()> {
    type TypedSelf = Assign<Type>;

    fn infer(
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
        let typed_expr = self.expr.infer(subs, var_state, env)?;
        let expr_ty = typed_expr.ty;
        let mut expr_expr = typed_expr.value;
        let (mut_var, typed_place) = self.place.infer_with_mut(subs, var_state, env)?;
        let place_ty = typed_place.ty;
        let mut place_expr = typed_place.value;
        if let Some(mut_var) = mut_var {
            let mut mut_subs = Subs::new();
            MutType::Var(mut_var).unify_with(MutType::Mut, &mut mut_subs, var_state)?;
            substitute_hir(&mut expr_expr, &mut_subs)?;
            substitute_hir(&mut place_expr, &mut_subs)?;
            subs.compose_with(mut_subs)?;
        }
        let mut place_subs = Subs::new();
        place_ty.unify_with(expr_ty, &mut place_subs, var_state)?;
        substitute_hir(&mut expr_expr, &place_subs)?;
        substitute_hir(&mut place_expr, &place_subs)?;
        subs.compose_with(place_subs)?;
        Ok(Typed {
            ty: unit(),
            value: Assign {
                place: place_expr,
                expr: expr_expr,
            },
        })
    }
}
impl Inferable for Box<[Assign<()>]> {
    type TypedSelf = Box<[Assign<Type>]>;

    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let assigns: Vec<_> = self.into();
        let assigns = assigns
            .into_iter()
            .map(|assign| assign.infer(subs, var_state, env).map(|typed| typed.value))
            .collect::<Result<Vec<_>, _>>()?
            .into();
        Ok(Typed {
            ty: unit(),
            value: assigns,
        })
    }
}
impl Inferable for Jump<()> {
    type TypedSelf = Jump<Type>;

    fn infer(
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
                    Some(expr) => expr.infer(subs, var_state, env)?.map(Some),
                    None => Typed {
                        ty: unit(),
                        value: None,
                    },
                };
                let operand_ty = typed_expr.ty;
                let mut operand_expr = typed_expr.value;
                let mut return_subs = Subs::new();
                operand_ty.unify_with(
                    keyword!("return").infer(subs, var_state, env)?.ty,
                    &mut return_subs,
                    var_state,
                )?;
                substitute_hir(&mut operand_expr, &return_subs)?;
                subs.compose_with(return_subs)?;
                Jump::Return(operand_expr.map(Box::new))
            }
        };
        let var = var_state.new_var();
        Ok(Typed {
            ty: Type::Var(var),
            value: typed,
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
        Statement::Declare(declare) => {
            let typed_expr = declare.expr.infer(subs, var_state, env)?;
            let operand_ty = typed_expr.ty;
            let mut operand_expr = typed_expr.value;
            let typed_pattern = declare.pattern.infer(None, var_state, env)?;
            let pattern_ty = typed_pattern.ty;
            let mut pattern_expr = typed_pattern.value;
            let mut more_subs = Subs::new();
            operand_ty.unify_with(pattern_ty, &mut more_subs, var_state)?;
            env.substitute(&more_subs)?;
            substitute_hir(&mut operand_expr, &more_subs)?;
            substitute_hir(&mut pattern_expr, &more_subs)?;
            subs.compose_with(more_subs)?;
            Statement::Declare(Declare {
                pattern: pattern_expr,
                expr: operand_expr,
            })
        }
        Statement::FunDeclare(_fun) => todo!(),
        // Statement::FunDeclare(fun) => {
        //     let var = Var::new_bare(fun.ident.clone());
        //     env.remove(var.clone());
        //     let mut ty = Type::Cons(Cons::Fun(
        //         Box::new(Type::Var(var_state.new_var())),
        //         Box::new(Type::Var(var_state.new_var())),
        //     ));
        //     env.insert(
        //         var.clone(),
        //         SchemeMut {
        //             is_mut: false,
        //             scheme: Scheme {
        //                 for_all: HashSet::new(),
        //                 ty: ty.clone(),
        //             },
        //         },
        //     );
        //     let typed_fun = fun.fun.infer(subs, var_state, env)?;
        //     let mut more_subs = Subs::new();
        //     typed_fun
        //         .ty
        //         .unify_with(ty.clone(), &mut more_subs, var_state)?;
        //     ty.substitute(&more_subs)?;
        //     subs.compose_with(more_subs)?;
        //     env.insert(
        //         var,
        //         SchemeMut {
        //             is_mut: false,
        //             scheme: env.generalize(ty.clone()),
        //         },
        //     );
        //     Statement::FunDeclare(FunDeclare {
        //         ident: fun.ident,
        //         fun: typed_fun.value,
        //         ty,
        //     })
        // }
        Statement::Expr(expr) => Statement::Expr(expr.infer(subs, var_state, env)?.value),
    };
    Ok(typed)
}
impl Inferable for Block<()> {
    type TypedSelf = Block<Type>;

    fn infer(
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
            Some(expr) => expr.infer(subs, var_state, &env)?.map(Some),
            None => Typed {
                ty: unit(),
                value: None,
            },
        };
        let mut ty = typed_expr.ty;
        ty.substitute(&more_subs)?;
        subs.compose_with(more_subs)?;
        Ok(Typed {
            ty,
            value: Block {
                statement: typed_statement.into(),
                expr: typed_expr.value.map(Box::new),
            },
        })
    }
}
impl Inferable for If<()> {
    type TypedSelf = If<Type>;

    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError>
    where
        Self: Sized,
    {
        let typed_condition = self.condition.infer(subs, var_state, env)?;
        let condition_ty = typed_condition.ty;
        let mut condition_expr = typed_condition.value;
        let typed_body = self.body.infer(subs, var_state, env)?;
        let mut body_ty = typed_body.ty;
        let mut body_expr = typed_body.value;
        let typed_else = match self.else_part {
            Some(else_part) => else_part.infer(subs, var_state, env)?.map(Some),
            None => Typed {
                ty: unit(),
                value: None,
            },
        };
        let else_ty = typed_else.ty;
        let mut else_expr = typed_else.value;
        let mut condition_subs = Subs::new();
        condition_ty.unify_with(Type::Cons(Cons::Bool), &mut condition_subs, var_state)?;
        substitute_hir(&mut condition_expr, &condition_subs)?;
        substitute_hir(&mut body_expr, &condition_subs)?;
        substitute_hir(&mut else_expr, &condition_subs)?;
        subs.compose_with(condition_subs)?;
        let mut body_else_subs = Subs::new();
        body_ty
            .clone()
            .unify_with(else_ty, &mut body_else_subs, var_state)?;
        body_ty.substitute(&body_else_subs)?;
        substitute_hir(&mut condition_expr, &body_else_subs)?;
        substitute_hir(&mut body_expr, &body_else_subs)?;
        substitute_hir(&mut else_expr, &body_else_subs)?;
        subs.compose_with(body_else_subs)?;
        Ok(Typed {
            ty: body_ty,
            value: If {
                condition: Box::new(condition_expr),
                body: body_expr,
                else_part: else_expr.map(Box::new),
            },
        })
    }
}
impl Inferable for ControlFlow<()> {
    type TypedSelf = ControlFlow<Type>;

    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let typed = match self {
            Self::Block(block) => block.infer(subs, var_state, env)?.map(ControlFlow::Block),
            Self::If(if_expr) => if_expr.infer(subs, var_state, env)?.map(ControlFlow::If),
            Self::For(_) => todo!(),
            Self::While(_) => todo!(),
            Self::Loop(_) => todo!(),
            Self::Match(_) => todo!(),
        };
        Ok(typed)
    }
}
impl Inferable for ExprKind<()> {
    type TypedSelf = ExprKind<Type>;

    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let ty_expr = match self {
            Self::Literal(literal) => literal.infer(subs, var_state, env)?.map(ExprKind::Literal),
            Self::Place(place) => place.infer(subs, var_state, env)?.map(ExprKind::Place),
            Self::Array(elements) => elements.infer(subs, var_state, env)?.map(ExprKind::Array),
            Self::ArrayRange(range) => range.infer(subs, var_state, env)?.map(ExprKind::ArrayRange),
            Self::Tag(tag) => tag.infer(subs, var_state, env)?.map(ExprKind::Tag),
            Self::Record(record) => record.infer(subs, var_state, env)?.map(ExprKind::Record),
            Self::Tuple(tuple) => tuple.infer(subs, var_state, env)?.map(ExprKind::Tuple),
            Self::Unit => Typed {
                ty: unit(),
                value: ExprKind::Unit,
            },
            Self::Splat(splat) => {
                let splat = splat.infer(subs, var_state, env)?;
                let operand_ty = splat.ty;
                let mut operand_expr = splat.value;
                let var = var_state.new_var();
                let mut ty = Type::Cons(Cons::RecordTuple(OrderedAnd::Row(
                    Vec::new(),
                    var.clone(),
                    Vec::new(),
                )));
                let mut operand_subs = Subs::new();
                operand_ty.unify_with(Type::Var(var), &mut operand_subs, var_state)?;
                ty.substitute(&operand_subs)?;
                substitute_hir(&mut operand_expr, &operand_subs)?;
                subs.compose_with(operand_subs)?;
                Typed {
                    ty,
                    value: ExprKind::Splat(Box::new(operand_expr)),
                }
            }
            Self::Unary(unary) => unary.infer(subs, var_state, env)?.map(ExprKind::Unary),
            Self::Binary(binary) => binary.infer(subs, var_state, env)?.map(ExprKind::Binary),
            Self::Fun(_fun) => todo!(),
            Self::Call(call) => call.infer(subs, var_state, env)?.map(ExprKind::Call),
            Self::Assign(assigns) => assigns.infer(subs, var_state, env)?.map(ExprKind::Assign),
            Self::Jump(jump) => jump.infer(subs, var_state, env)?.map(ExprKind::Jump),
            Self::ControlFlow(control_flow) => control_flow
                .infer(subs, var_state, env)?
                .map(ExprKind::ControlFlow),
        };
        Ok(ty_expr)
    }
    fn infer_with_mut(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<(Option<Var>, Typed<ExprKind<Type>>), TypeError> {
        let mut_typed = if let Self::Place(place) = self {
            let (mut_var, typed) = place.infer_with_mut(subs, var_state, env)?;
            (mut_var, typed.map(ExprKind::Place))
        } else {
            (None, self.infer(subs, var_state, env)?)
        };
        Ok(mut_typed)
    }
}
impl Inferable for Expr<()> {
    type TypedSelf = Expr<Type>;
    fn infer(
        self,
        subs: &mut Subs,
        var_state: &mut VarState,
        env: &Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError>
    where
        Self: Sized,
    {
        let typed = self.expr.infer(subs, var_state, env)?;
        let ty = typed.ty.clone();
        Ok(Typed {
            ty,
            value: Expr {
                expr: typed.value,
                ty: typed.ty,
            },
        })
    }
}
