use std::collections::HashMap;
use typed_arena::Arena;

#[derive(Debug, PartialEq, Eq)]
pub enum Type<'a> {
    Int,
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

pub type Context<'a> = HashMap<Id, &'a Type<'a>>;

pub fn infer<'a>(
    arena: &'a Arena<Type<'a>>,
    ctx: &Context<'a>,
    term: &'a Term<'a>,
) -> Result<&'a Type<'a>, Error<'a>> {
    match *term {
        Term::Var(id) => Ok(*ctx.get(&id).ok_or(Error::UnknownVariable(id))?),
        Term::Lam(arg_id, arg_ty, body) => {
            let mut ctx2 = ctx.clone();
            ctx2.insert(arg_id, arg_ty);
            let result_ty = infer(arena, &ctx2, body)?;
            Ok(arena.alloc(Type::Fun(arg_ty, result_ty)))
        }
        Term::App(fun, arg) => {
            let fun_ty = infer(arena, ctx, fun)?;
            let arg_ty = infer(arena, ctx, arg)?;
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
        let mut ctx: Context = HashMap::new();
        ctx.insert(1, arena.alloc(Int));
        assert_eq!(infer(&arena, &ctx, &Var(1)), Ok(&Int));
    }

    #[test]
    fn lam() {
        let arena = Arena::new();
        assert_eq!(
            infer(&arena, &HashMap::new(), &Lam(1, &Int, &Var(1))),
            Ok(&Fun(&Int, &Int))
        );
    }

    #[test]
    fn app() {
        let arena = Arena::new();
        let mut ctx: Context = HashMap::new();
        ctx.insert(1, arena.alloc(Fun(arena.alloc(Int), arena.alloc(Int))));
        ctx.insert(2, arena.alloc(Int));
        assert_eq!(infer(&arena, &ctx, &App(&Var(1), &Var(2))), Ok(&Int));
    }
}
