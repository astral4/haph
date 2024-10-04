//! # haph
//!
//! Hasher-agnostic static hashmaps

#![cfg_attr(not(test), no_std)]

mod generate;

extern crate alloc;

use alloc::vec::Vec;
use core::borrow::Borrow;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use foldhash::{HashSet, HashSetExt};
use num_traits::bounds::UpperBounded;
use num_traits::{AsPrimitive, Unsigned, WrappingAdd, WrappingMul, Zero};
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::{Rng, SeedableRng};
use usize_cast::IntoUsize;

pub trait MapHasher<S, H>: Hasher
where
    H: 'static + UpperBounded + Unsigned + IntoUsize + Zero + Copy + WrappingMul + WrappingAdd,
{
    fn new_with_seed(seed: &S) -> Self;

    fn finish_triple(&self) -> (H, H, H);
}

pub struct Map<M, S, H, K, V> {
    seed: S,
    displacements: Vec<(H, H)>,
    entries: Vec<(K, V)>,
    _marker: PhantomData<M>,
}

impl<M, S, H, K, V> Map<M, S, H, K, V> {
    pub fn new<R>(entries: Vec<(K, V)>) -> Self
    where
        R: SeedableRng + Rng,
        K: Eq + Hash,
        M: MapHasher<S, H>,
        H: 'static + UpperBounded + Unsigned + IntoUsize + Zero + Copy + WrappingMul + WrappingAdd,
        Standard: Distribution<S>,
        usize: AsPrimitive<H>,
    {
        assert!(
            entries.len() <= H::max_value().into_usize(),
            "cannot have more entries than possible hash values"
        );

        let keys: Vec<_> = entries.iter().map(|entry| &entry.0).collect();

        assert!(!has_duplicates(&keys), "duplicate key present");

        let (seed, state) = generate::generate::<R, _, M, _, _>(&keys);

        let mut entries = entries;
        sort_by_indices(&mut entries, state.indices);

        Self {
            seed,
            displacements: state.displacements,
            entries,
            _marker: PhantomData,
        }
    }
}

#[inline]
fn has_duplicates<T: Eq + Hash>(items: &[T]) -> bool {
    let mut set = HashSet::with_capacity(items.len());

    for item in items {
        if !set.insert(item) {
            return true;
        }
    }

    false
}

#[inline]
fn sort_by_indices<T>(data: &mut [T], mut indices: Vec<usize>) {
    for idx in 0..data.len() {
        if indices[idx] != idx {
            let mut current_idx = idx;
            loop {
                let target_idx = indices[current_idx];
                indices[current_idx] = current_idx;
                if indices[target_idx] == target_idx {
                    break;
                }
                data.swap(current_idx, target_idx);
                current_idx = target_idx;
            }
        }
    }
}

impl<M, S, H, K, V> Map<M, S, H, K, V> {
    pub fn get_entry<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        Q: Hash + Eq + ?Sized,
        K: Borrow<Q>,
        M: MapHasher<S, H>,
        H: 'static + UpperBounded + Unsigned + IntoUsize + Zero + Copy + WrappingMul + WrappingAdd,
    {
        if self.displacements.is_empty() {
            return None;
        }

        let hashes = generate::hash::<_, M, _, _>(key, &self.seed);
        let (d1, d2) = self.displacements[hashes.0.into_usize() % self.displacements.len()];
        let index =
            generate::displace(hashes.1, hashes.2, d1, d2).into_usize() % self.entries.len();
        let entry = &self.entries[index];

        if entry.0.borrow() == key {
            Some((&entry.0, &entry.1))
        } else {
            None
        }
    }
}
