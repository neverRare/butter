use std::collections::HashSet;

use hir::pattern::{self, Pattern};

use crate::{
    ty::{Env, Scheme, SchemeMut, VarState},
    Cons, MutType, Type, TypeError, Typed, Var,
};

pub(super) trait InferablePattern {
    type TypedSelf;

    fn infer(
        self,
        var_state: &mut VarState,
        env: &mut Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError>;
}
impl InferablePattern for pattern::Var<()> {
    type TypedSelf = pattern::Var<Type>;

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
                    ty: ty.clone(),
                },
            },
        );
        Ok(Typed {
            ty: Type::Var(var),
            value: pattern::Var {
                ident: self.ident,
                mutable: self.mutable,
                bind_to_ref: self.bind_to_ref,
                ty,
            },
        })
    }
}
impl InferablePattern for Pattern<()> {
    type TypedSelf = Pattern<Type>;

    fn infer(
        self,
        var_state: &mut VarState,
        env: &mut Env,
    ) -> Result<Typed<Self::TypedSelf>, TypeError> {
        let typed = match self {
            Pattern::Var(var) => var.infer(var_state, env)?.map(Pattern::Var),
            Pattern::True => todo!(),
            Pattern::False => todo!(),
            Pattern::UInt(_) => todo!(),
            Pattern::Int(_) => todo!(),
            Pattern::Ignore => todo!(),
            Pattern::Record(_) => todo!(),
            Pattern::Tuple(_) => todo!(),
            Pattern::Array(_) => todo!(),
            Pattern::Tag(_) => todo!(),
            Pattern::Ref(_) => todo!(),
        };
        Ok(typed)
    }
}
