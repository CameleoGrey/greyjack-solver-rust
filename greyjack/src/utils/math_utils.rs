// src/utils/math_utils.rs

use rand::{rngs::SmallRng, seq::SliceRandom, Rng, SeedableRng};
use rand_distr::{Distribution, Uniform};
use std::{collections::HashSet, hash::Hash};

/// Creates a standard random number generator from entropy.
/// This should be created once and passed around where needed.
pub fn create_rng() -> SmallRng {
    SmallRng::from_entropy()
}

pub fn rint(x: f64) -> f64 {
    if (x - x.floor()).abs() < (x.ceil() - x).abs() {
        x.floor()
    } else {
        x.ceil()
    }
}

pub fn round(value: f64, precision: u64) -> f64 {
    let multiplier = (10.0 as f64).powf(precision as f64);
    value.floor() + ((value - value.floor()) * multiplier).floor() / multiplier
}
