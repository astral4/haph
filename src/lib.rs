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

pub trait MapHasher<S, H>: Hasher
where
    H: 'static + UpperBounded + Unsigned + IntoUsize + Zero + Copy + WrappingMul + WrappingAdd,
{
    fn new_with_seed(seed: &S) -> Self;

    fn finish_triple(&self) -> (H, H, H);
}
