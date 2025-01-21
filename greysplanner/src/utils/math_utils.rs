
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use rand_distr::{Distribution, Uniform};
use std::{collections::HashSet, hash::Hash};

pub fn rint(x: f64) -> f64 {
    if (x - x.floor()).abs() < (x.ceil() - x).abs() {x.floor()} else {x.ceil()}
}

pub fn round(value: f64, precision: u64) -> f64 {
    let multiplier = (10.0 as f64).powf(precision as f64);
    value.floor() + ((value - value.floor()) * multiplier).floor() / multiplier
}

pub fn get_random_id(start_id: usize, end_id: usize) -> usize {
    Uniform::new(start_id, end_id).sample(&mut StdRng::from_entropy())
}

pub fn choice<T>(objects: &Vec<T>, n: usize, replace: bool) -> Vec<T>
where T: Clone {
    if replace == true {
        choice_with_replacement(objects, n)
    } else {
        choice_without_replacement(objects, n)
    }
}

fn choice_with_replacement<T>(objects: &Vec<T>, n: usize) -> Vec<T>
where T: Clone {
    
    let objects_count = objects.len();
    let chosen_objects: Vec<T> = (0..n).into_iter().map(|i| objects[get_random_id(0, objects_count)].clone()).collect();
    return chosen_objects;
}

fn choice_without_replacement<T>(objects: &Vec<T>, n: usize) -> Vec<T>
where T: Clone {

    if n > objects.len() {
        panic!("There are less objects tnan can be chosen from collection without replacement");
    }
    
    let mut random_ids:Vec<usize> = (0..objects.len()).collect();
    random_ids.shuffle(&mut StdRng::from_entropy());
    let chosen_objects: Vec<T> = (0..n).into_iter().map(|i| objects[random_ids[i]].clone()).collect();

    return chosen_objects;
}
