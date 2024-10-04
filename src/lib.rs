//! # haph
//!
//! Hasher-agnostic static hashmaps

#![cfg_attr(not(test), no_std)]

mod generate;

extern crate alloc;

use core::hash::Hasher;
use num_traits::bounds::UpperBounded;
use num_traits::{Unsigned, WrappingAdd, WrappingMul, Zero};
use usize_cast::IntoUsize;

pub trait MapHasher<S>: Hasher {
    type Hash: 'static
        + UpperBounded
        + Unsigned
        + IntoUsize
        + Zero
        + Copy
        + WrappingMul
        + WrappingAdd;

    fn new_with_seed(seed: &S) -> Self;

    fn finish_triple(&self) -> Hashes<Self, S>;
}

pub type Hashes<M, S> = (
    <M as MapHasher<S>>::Hash,
    <M as MapHasher<S>>::Hash,
    <M as MapHasher<S>>::Hash,
);
