// src/utils/math_utils.rs

use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use rand_distr::{Distribution, Uniform};
use std::{collections::HashSet, hash::Hash};

/// Creates a standard random number generator from entropy.
/// This should be created once and passed around where needed.
pub fn create_rng() -> StdRng {
    StdRng::from_entropy()
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

/// Gets a random integer within a specified range using a provided RNG.
pub fn get_random_id(start_id: usize, end_exclusive: usize, rng: &mut StdRng) -> usize {
    Uniform::new(start_id, end_exclusive).sample(rng)
}

/// Chooses a random sample of items from a vector, with or without replacement.
pub fn choice<T>(objects: &Vec<T>, n: usize, replace: bool, rng: &mut StdRng) -> Vec<T>
where
    T: Clone,
{
    if replace {
        choice_with_replacement(objects, n, rng)
    } else {
        choice_without_replacement(objects, n, rng)
    }
}

/// Helper for choosing items with replacement.
fn choice_with_replacement<T>(objects: &Vec<T>, n: usize, rng: &mut StdRng) -> Vec<T>
where
    T: Clone,
{
    let objects_count = objects.len();
    let chosen_objects: Vec<T> = (0..n)
        .into_iter()
        .map(|_| objects[get_random_id(0, objects_count, rng)].clone())
        .collect();
    chosen_objects
}

/// Helper for choosing items without replacement.
fn choice_without_replacement<T>(objects: &Vec<T>, n: usize, rng: &mut StdRng) -> Vec<T>
where
    T: Clone,
{
    if n > objects.len() {
        panic!("There are fewer objects than can be chosen from the collection without replacement");
    }

    let mut random_ids: Vec<usize> = (0..objects.len()).collect();
    random_ids.shuffle(rng);
    let chosen_objects: Vec<T> = (0..n)
        .into_iter()
        .map(|i| objects[random_ids[i]].clone())
        .collect();

    chosen_objects
}
