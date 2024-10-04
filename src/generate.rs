use super::{Hashes, MapHasher};
use alloc::vec;
use alloc::vec::Vec;
use core::hash::Hash;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, Zero};
use rand::distributions::{Distribution, Standard};
use rand::{Rng, SeedableRng};
use usize_cast::IntoUsize;

const FIXED_SEED: u64 = 42;

const LAMBDA: usize = 5;

pub(crate) struct MapState<S, M: MapHasher<S>> {
    pub(crate) displacements: Vec<(M::Hash, M::Hash)>,
    pub(crate) indices: Vec<usize>,
}

struct Bucket {
    index: usize,
    keys: Vec<usize>,
}

impl Bucket {
    fn new(index: usize) -> Self {
        Self {
            index,
            keys: Vec::new(),
        }
    }
}

pub(crate) fn generate<R, T, M, S>(entries: &[T]) -> (S, MapState<S, M>)
where
    R: SeedableRng + Rng,
    T: Hash,
    M: MapHasher<S>,
    Standard: Distribution<S>,
    usize: AsPrimitive<M::Hash>,
{
    R::seed_from_u64(FIXED_SEED)
        .sample_iter(Standard)
        .find_map(|seed| {
            let hashes: Vec<_> = entries
                .iter()
                .map(|entry| hash::<_, M, _>(entry, &seed))
                .collect();
            try_generate(&hashes).map(|s| (seed, s))
        })
        .expect("failed to obtain PHF")
}

fn try_generate<M, S>(hashes: &[Hashes<M, S>]) -> Option<MapState<S, M>>
where
    M: MapHasher<S>,
    usize: AsPrimitive<M::Hash>,
{
    let table_len = hashes.len();
    let num_buckets = table_len.div_ceil(LAMBDA);

    let mut buckets: Vec<_> = (0..num_buckets).map(Bucket::new).collect();

    for (i, hash) in hashes.iter().enumerate() {
        buckets[hash.0.into_usize() % num_buckets].keys.push(i);
    }
    buckets.sort_by(|a, b| Ord::cmp(&a.keys.len(), &b.keys.len()).reverse());

    let mut displacements = vec![(M::Hash::zero(), M::Hash::zero()); num_buckets];
    let mut map = vec![None; table_len];
    let mut try_map = vec![0u64; table_len];
    let mut generation = 0;
    let mut values_to_add = Vec::with_capacity(LAMBDA);

    'buckets: for bucket in &buckets {
        for d1 in 0..table_len {
            'disps: for d2 in 0..table_len {
                let (d1, d2) = (d1.as_(), d2.as_());
                values_to_add.clear();
                generation += 1;

                for &key in &bucket.keys {
                    let index =
                        displace(hashes[key].1, hashes[key].2, d1, d2).into_usize() % table_len;

                    if map[index].is_some() || try_map[index] == generation {
                        continue 'disps;
                    }

                    try_map[index] = generation;
                    values_to_add.push((index, key));
                }

                displacements[bucket.index] = (d1, d2);
                for &(index, key) in &values_to_add {
                    map[index] = Some(key);
                }
                continue 'buckets;
            }
        }
        return None;
    }

    Some(MapState {
        displacements,
        indices: map.into_iter().map(Option::unwrap).collect(),
    })
}

// pub(crate) fn hash<T: Hash, M: MapHasher<S>, S>(x: T, seed: &S) -> Hashes<M, S> {
pub(crate) fn hash<T, M, S>(x: T, seed: &S) -> Hashes<M, S>
where
    T: Hash,
    M: MapHasher<S>,
{
    let mut hasher = M::new_with_seed(seed);
    x.hash(&mut hasher);
    hasher.finish_triple()
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn displace<T: WrappingMul + WrappingAdd>(f1: T, f2: T, d1: T, d2: T) -> T {
    f1.wrapping_mul(&d1).wrapping_add(&f2).wrapping_add(&d2)
}
