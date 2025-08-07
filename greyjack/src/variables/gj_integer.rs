use std::{cmp::Ordering::*, collections::HashMap};
use rand::rngs::SmallRng;
use rand_distr::{Normal, Distribution, Uniform};
use crate::utils::math_utils;

#[derive(Debug, Clone)]
pub struct GJInteger {
    pub name: String,
    pub initial_value: Option<f64>,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub frozen: bool,
    pub random_generator: SmallRng,
    pub uniform_distribution: Uniform<i64>,
    pub normal_distribution: Option<Normal<f64>>,
    pub semantic_groups: Vec<String>
}

impl GJInteger {
    pub fn new(initial_value: Option<i64>, lower_bound: i64, upper_bound: i64, frozen: bool, semantic_groups: Option<Vec<String>>) -> Self {
            
            let normal_distribution = initial_value.map(|x| Normal::new(x as f64, 0.1).unwrap());
            
            let casted_initial_value = initial_value.map(|x| x as f64);

            let mut current_semantic_groups: Vec<String> = Vec::new();
            match semantic_groups {
                None => current_semantic_groups.push("common".to_string()),
                Some(groups) => {
                    current_semantic_groups.extend(groups);
                },
            }

            GJInteger {
                name: "".to_string(),
                initial_value: casted_initial_value,
                lower_bound: lower_bound as f64,
                upper_bound: upper_bound as f64,
                frozen,
                random_generator: math_utils::create_rng(),
                uniform_distribution: Uniform::new_inclusive(lower_bound, upper_bound),
                normal_distribution,
                semantic_groups: current_semantic_groups
            }
        }
}

impl GJInteger {

    pub fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    pub fn inverse_transform(&self, value: f64) -> i64 {
        self.fix(value) as i64
    }

    pub fn fix(&self, value: f64) -> f64 {
        if self.frozen {
            return self.initial_value.expect("Frozen value must be initialized");
        }

        let clamped_value = value.clamp(self.lower_bound, self.upper_bound);
        math_utils::rint(clamped_value)
    }

    pub fn sample(&mut self) -> f64 {
        if self.frozen {
            return self.initial_value.expect("Frozen value must be initialized");
        }

        self.uniform_distribution.sample(&mut self.random_generator) as f64
    }

    pub fn get_initial_value(&mut self) -> f64 {
        match self.initial_value {
            None => self.sample(),
            Some(x) => {
                if self.frozen {
                    return x;
                }
                x
            }
        }
    }
}