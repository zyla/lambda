use crate::context;
use std::collections::HashMap;
use std::ptr;
use typed_arena::Arena;

pub type TyVar = usize;

#[derive(Debug, PartialEq, Eq)]
pub enum Type<'a> {
    Int,
    Bool,
    Var(TyVar),
    Fun(&'a Type<'a>, &'a Type<'a>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct PolyType<'a> {
    foralls: Vec<TyVar>,
    ty: &'a Type<'a>,
}

pub type Id = usize;

#[derive(Debug, PartialEq, Eq)]
pub enum Term<'a> {
    Var(Id),
    Let(Id, &'a Term<'a>, &'a Term<'a>),
    Lam(Id, &'a Term<'a>),
    App(&'a Term<'a>, &'a Term<'a>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error<'a> {
    UnknownVariable(Id),
    ApplyingNonFunction {
        fun_ty: &'a Type<'a>,
        arg_ty: &'a Type<'a>,
    },
    ArgTypeMismatch {
        expected: &'a Type<'a>,
        actual: &'a Type<'a>,
    },
}

pub type Context<'a> = context::Context<Id, &'a PolyType<'a>>;

pub fn infer<'a>(
    arena: &'a Arena<Type<'a>>,
    ctx: &Context<'a>,
    term: &'a Term<'a>,
) -> Result<&'a Type<'a>, Error<'a>> {
    let mut ctx = ctx.clone();
    infer_mut(arena, &mut ctx, term)
}

struct TyEnv<'a> {
    tyvar_union: Vec<TyVar>,
    tyvar_ty: Vec<Option<&'a Type<'a>>>,
}

impl<'a> TyEnv<'a> {
    fn new() -> Self {
        Self {
            tyvar_union: Default::default(),
            tyvar_ty: Default::default(),
        }
    }

    fn fresh(&mut self) -> TyVar {
        let id = self.tyvar_union.len();
        self.tyvar_union.push(id);
        self.tyvar_ty.push(None);
        id
    }
}

fn instantiate<'a>(
    arena: &'a Arena<Type<'a>>,
    tyenv: &mut TyEnv,
    polyty: &'a PolyType<'a>,
) -> &'a Type<'a> {
    let mapping: HashMap<TyVar, TyVar> = polyty
        .foralls
        .iter()
        .map(|from| (*from, tyenv.fresh()))
        .collect();
    subst(arena, &mapping, polyty.ty)
}

fn subst<'a>(
    arena: &'a Arena<Type<'a>>,
    mapping: &HashMap<TyVar, TyVar>,
    ty: &'a Type<'a>,
) -> &'a Type<'a> {
    match *ty {
        Type::Int => ty,
        Type::Bool => ty,
        Type::Var(tv) => match mapping.get(&tv) {
            Some(tv) => arena.alloc(Type::Var(*tv)),
            None => ty,
        },
        Type::Fun(arg, ret) => {
            let new_arg = subst(arena, mapping, arg);
            let new_ret = subst(arena, mapping, ret);
            // avoid allocation if the type is unchanged
            if ptr::eq(arg, new_arg) && ptr::eq(ret, new_ret) {
                ty
            } else {
                arena.alloc(Type::Fun(new_arg, new_ret))
            }
        }
    }
}

pub fn infer_mut<'a>(
    arena: &'a Arena<Type<'a>>,
    ctx: &mut Context<'a>,
    term: &'a Term<'a>,
) -> Result<&'a Type<'a>, Error<'a>> {
    unimplemented!();
}

#[cfg(test)]
mod tests {
    use super::{Term::*, Type::*, *};

    #[test]
    fn test_instantiate_1() {
        let arena = Arena::new();
        let mut tyenv = TyEnv::new();
        assert_eq!(0, tyenv.fresh());
        assert_eq!(
            instantiate(
                &arena,
                &mut tyenv,
                &PolyType {
                    foralls: vec![0],
                    ty: &Type::Var(0)
                }
            ),
            &Type::Var(1)
        );
    }

    #[test]
    fn test_instantiate_2() {
        let arena = Arena::new();
        let mut tyenv = TyEnv::new();
        assert_eq!(0, tyenv.fresh());
        assert_eq!(1, tyenv.fresh());
        assert_eq!(
            instantiate(
                &arena,
                &mut tyenv,
                &PolyType {
                    foralls: vec![1],
                    ty: &Type::Var(0)
                }
            ),
            &Type::Var(0)
        );
    }

    #[test]
    fn test_instantiate_3() {
        let arena = Arena::new();
        let mut tyenv = TyEnv::new();
        assert_eq!(0, tyenv.fresh());
        assert_eq!(
            instantiate(
                &arena,
                &mut tyenv,
                &PolyType {
                    foralls: vec![0],
                    ty: &Type::Fun(&Type::Int, &Type::Var(0))
                }
            ),
            &Type::Fun(&Type::Int, &Type::Var(1))
        );
    }
}
