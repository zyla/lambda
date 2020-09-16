#![feature(test)]
extern crate test;

use std::collections::HashMap;
use typed_arena::Arena;

use lambda::stlc::{Term::*, Type::*, *, *};

// #[allow(soft_unstable)]
#[bench]
fn lambdas(b: &mut test::bench::Bencher) {
    let ty_arena = Arena::new();
    let term_arena = Arena::new();
    let mut term = term_arena.alloc(Var(1));
    for _ in 1..100 {
        term = term_arena.alloc(Lam(1, ty_arena.alloc(Int), term));
    }
    let ctx = HashMap::new();
    b.iter(|| {
        for _ in 1..1000 {
            let arena = Arena::new();
            test::black_box(infer(&arena, &ctx, &term));
        }
    })
}
