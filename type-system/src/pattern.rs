use crate::{
    expr::unit,
    ty::{Env, Scheme, SchemeMut, VarState},
    Cons, Keyed, MutType, Type, TypeError, Typed, Var,
};
use hir::pattern::{self, Pattern, PatternKind, TaggedPattern};
use std::{collections::HashSet, iter::once};

pub(super) trait InferablePattern {
    type TypedSelf;

    fn infer(
        self,
        mut_type: Option<Var>,
        var_state: &mut VarState,
        env: &mut Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError>;
}
impl InferablePattern for pattern::Var {
    type TypedSelf = pattern::Var;

    fn infer(
        self,
        mut_type: Option<Var>,
        var_state: &mut VarState,
        env: &mut Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let var = var_state.new_named(self.ident.clone());
        let mut ty = Type::Var(var.clone());
        if self.bind_to_ref {
            ty = Type::Cons(Cons::Ref(
                MutType::Var(mut_type.unwrap_or_else(|| var_state.new_var())),
                Box::new(ty),
            ));
        }
        env.insert(
            Var::new_bare(self.ident.clone()),
            SchemeMut {
                is_mut: self.mutable,
                scheme: Scheme {
                    for_all: HashSet::new(),
                    ty,
                },
            },
        );
        Ok(Typed {
            ty: Type::Var(var),
            value: pattern::Var {
                ident: self.ident,
                mutable: self.mutable,
                bind_to_ref: self.bind_to_ref,
            },
        })
    }
}
impl InferablePattern for TaggedPattern<()> {
    type TypedSelf = TaggedPattern<Type>;

    fn infer(
        self,
        mut_type: Option<Var>,
        var_state: &mut VarState,
        env: &mut Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let (pattern, ty) = match self.pattern {
            Some(pattern) => {
                let typed = pattern.infer(mut_type, var_state, env)?;
                (Some(typed.value), typed.ty)
            }
            None => (None, unit()),
        };
        Ok(Typed {
            ty: Type::Cons(Cons::Union(Keyed {
                fields: once((self.tag.clone(), ty)).collect(),
                rest: Some(var_state.new_var()),
            })),
            value: TaggedPattern {
                tag: self.tag,
                pattern: pattern.map(Box::new),
            },
        })
    }
}
impl InferablePattern for PatternKind<()> {
    type TypedSelf = PatternKind<Type>;

    fn infer(
        self,
        mut_type: Option<Var>,
        var_state: &mut VarState,
        env: &mut Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let typed = match self {
            PatternKind::Var(var) => var.infer(mut_type, var_state, env)?.map(PatternKind::Var),
            PatternKind::True => Typed {
                ty: Type::Cons(Cons::Bool),
                value: PatternKind::True,
            },
            PatternKind::False => Typed {
                ty: Type::Cons(Cons::Bool),
                value: PatternKind::False,
            },
            PatternKind::UInt(num) => Typed {
                ty: Type::Cons(Cons::Num),
                value: PatternKind::UInt(num),
            },
            PatternKind::Int(num) => Typed {
                ty: Type::Cons(Cons::Num),
                value: PatternKind::Int(num),
            },
            PatternKind::Ignore => Typed {
                ty: Type::Var(var_state.new_var()),
                value: PatternKind::Ignore,
            },
            PatternKind::Record(_) => todo!(),
            PatternKind::Tuple(_) => todo!(),
            PatternKind::Param(_) => todo!(),
            PatternKind::Array(_) => todo!(),
            PatternKind::Tag(tag) => {
                let typed = tag.infer(mut_type, var_state, env)?;
                Typed {
                    ty: typed.ty,
                    value: PatternKind::Tag(typed.value),
                }
            }
            PatternKind::Ref(pattern) => {
                let typed = pattern.infer(mut_type, var_state, env)?;
                let var = var_state.new_var();
                Typed {
                    ty: Type::Cons(Cons::Ref(MutType::Var(var), Box::new(typed.ty))),
                    value: PatternKind::Ref(Box::new(typed.value)),
                }
            }
        };
        Ok(typed)
    }
}
impl InferablePattern for Pattern<()> {
    type TypedSelf = Pattern<Type>;

    fn infer(
        self,
        mut_type: Option<Var>,
        var_state: &mut VarState,
        env: &mut Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let typed = self.pattern.infer(mut_type, var_state, env)?;
        let ty = typed.ty.clone();
        Ok(Typed {
            value: Pattern {
                pattern: typed.value,
                ty: typed.ty,
            },
            ty,
        })
    }
}
