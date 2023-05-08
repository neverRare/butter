use crate::{
    ty::{Env, Scheme, SchemeMut, VarState},
    Cons, MutType, Type, TypeError, Typed, Var,
};
use hir::pattern::{self, Pattern, PatternKind};
use std::collections::HashSet;

pub(super) trait InferablePattern {
    type TypedSelf;

    fn infer(
        self,
        var_state: &mut VarState,
        env: &mut Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError>;
}
impl InferablePattern for pattern::Var {
    type TypedSelf = pattern::Var;

    fn infer(
        self,
        var_state: &mut VarState,
        env: &mut Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let var = var_state.new_named(self.ident.clone());
        let mut ty = Type::Var(var.clone());
        if self.bind_to_ref {
            ty = Type::Cons(Cons::Ref(MutType::Var(var_state.new_var()), Box::new(ty)));
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
impl InferablePattern for PatternKind<()> {
    type TypedSelf = PatternKind<Type>;

    fn infer(
        self,
        var_state: &mut VarState,
        env: &mut Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let typed = match self {
            PatternKind::Var(var) => var.infer(var_state, env)?.map(PatternKind::Var),
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
            PatternKind::Tag(_) => todo!(),
            PatternKind::Ref(_) => todo!(),
        };
        Ok(typed)
    }
}
impl InferablePattern for Pattern<()> {
    type TypedSelf = Pattern<Type>;

    fn infer(
        self,
        var_state: &mut VarState,
        env: &mut Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let typed = self.pattern.infer(var_state, env)?;
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
