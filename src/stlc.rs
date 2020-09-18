use crate::context;
use typed_arena::Arena;

#[derive(Debug, PartialEq, Eq)]
pub enum Type<'a> {
    Int,
    Bool,
    Fun(&'a Type<'a>, &'a Type<'a>),
}

pub type Id = usize;

#[derive(Debug, PartialEq, Eq)]
pub enum Term<'a> {
    Var(Id),
    Lam(Id, &'a Type<'a>, &'a Term<'a>),
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

pub type Context<'a> = context::Context<Id, &'a Type<'a>>;

pub fn infer<'a>(
    arena: &'a Arena<Type<'a>>,
    ctx: &Context<'a>,
    term: &'a Term<'a>,
) -> Result<&'a Type<'a>, Error<'a>> {
    let mut ctx = ctx.clone();
    infer_mut(arena, &mut ctx, term)
}

pub fn infer_mut<'a>(
    arena: &'a Arena<Type<'a>>,
    ctx: &mut Context<'a>,
    term: &'a Term<'a>,
) -> Result<&'a Type<'a>, Error<'a>> {
    match *term {
        Term::Var(id) => Ok(*ctx.get(&id).ok_or(Error::UnknownVariable(id))?),
        Term::Lam(arg_id, arg_ty, body) => {
            let result_ty = infer_mut(arena, &mut ctx.with(arg_id, arg_ty), body)?;
            Ok(arena.alloc(Type::Fun(arg_ty, result_ty)))
        }
        Term::App(fun, arg) => {
            let fun_ty = infer_mut(arena, ctx, fun)?;
            let arg_ty = infer_mut(arena, ctx, arg)?;
            match fun_ty {
                Type::Fun(expected_arg_ty, result_ty) => {
                    if arg_ty != *expected_arg_ty {
                        return Err(Error::ArgTypeMismatch {
                            expected: expected_arg_ty,
                            actual: arg_ty,
                        });
                    }
                    Ok(result_ty)
                }
                _ => Err(Error::ApplyingNonFunction { fun_ty, arg_ty }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Term::*, Type::*, *};

    #[test]
    fn var() {
        let arena = Arena::new();
        let mut ctx: Context = Context::default();
        ctx.insert(1, arena.alloc(Int));
        assert_eq!(infer(&arena, &ctx, &Var(1)), Ok(&Int));
    }

    #[test]
    fn lam() {
        let arena = Arena::new();
        assert_eq!(
            infer(&arena, &Context::default(), &Lam(1, &Int, &Var(1))),
            Ok(&Fun(&Int, &Int))
        );
    }

    #[test]
    fn app() {
        let arena = Arena::new();
        let mut ctx: Context = Context::default();
        ctx.insert(1, arena.alloc(Fun(arena.alloc(Int), arena.alloc(Int))));
        ctx.insert(2, arena.alloc(Int));
        assert_eq!(infer(&arena, &ctx, &App(&Var(1), &Var(2))), Ok(&Int));
    }

    #[test]
    fn nested_lambda_1() {
        let arena = Arena::new();
        assert_eq!(
            infer(
                &arena,
                &Context::default(),
                &Lam(1, &Int, &Lam(2, &Bool, &Var(1)))
            ),
            Ok(&Fun(&Int, &Fun(&Bool, &Int)))
        );
    }

    #[test]
    fn nested_lambda_2() {
        let arena = Arena::new();
        assert_eq!(
            infer(
                &arena,
                &Context::default(),
                &Lam(1, &Int, &Lam(2, &Bool, &Var(2)))
            ),
            Ok(&Fun(&Int, &Fun(&Bool, &Bool)))
        );
    }

    #[test]
    fn shadowing() {
        let arena = Arena::new();
        assert_eq!(
            infer(
                &arena,
                &Context::default(),
                &Lam(1, &Int, &Lam(1, &Bool, &Var(1)))
            ),
            Ok(&Fun(&Int, &Fun(&Bool, &Bool)))
        );
    }
}
