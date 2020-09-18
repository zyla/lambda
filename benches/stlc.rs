#![feature(test)]
extern crate test;

use typed_arena::Arena;

use lambda::stlc::{Term::*, Type::*, *};

#[bench]
fn lambdas(b: &mut test::bench::Bencher) {
    let ty_arena = Arena::new();
    let term_arena = Arena::new();
    let mut term = term_arena.alloc(Var(1));
    for _ in 1..100 {
        term = term_arena.alloc(Lam(1, ty_arena.alloc(Int), term));
    }
    let ctx = Context::default();
    b.iter(|| {
        for _ in 1..1000 {
            let arena = Arena::new();
            test::black_box(infer(&arena, &ctx, &term).unwrap());
        }
    })
}
