use std::cmp::Ordering::*;
use rand::rngs::SmallRng;
use rand_distr::{Normal, Distribution, Uniform};
use crate::utils::math_utils;

#[derive(Debug, Clone)]
pub struct GJFloat {
    pub name: String,
    pub initial_value: Option<f64>,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub frozen: bool,
    pub random_generator: SmallRng,
    pub uniform_distribution: Uniform<f64>,
    pub normal_distribution: Option<Normal<f64>>,
    pub semantic_groups: Vec<String>
}

impl GJFloat {
    pub fn new(initial_value: Option<f64>, 
        lower_bound: f64, upper_bound: f64, frozen: bool, semantic_groups: Option<Vec<String>>) -> Self {
            
            let normal_distribution = initial_value.map(|x| Normal::new(x, 0.1).unwrap());

            let mut current_semantic_groups: Vec<String> = Vec::new();
            match semantic_groups {
                None => current_semantic_groups.push("common".to_string()),
                Some(groups) => {
                    current_semantic_groups.extend(groups);
                },
            }

            GJFloat {
                name: "".to_string(),
                initial_value,
                lower_bound,
                upper_bound,
                frozen,
                random_generator: math_utils::create_rng(),
                uniform_distribution: Uniform::new_inclusive(lower_bound, upper_bound),
                normal_distribution,
                semantic_groups: current_semantic_groups
            }
        }
}

impl GJFloat {

    pub fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    pub fn inverse_transform(&self, value: f64) -> f64 {
        self.fix(value)
    }

    pub fn fix(&self, value: f64) -> f64 {
        if self.frozen {
            return self.initial_value.expect("Frozen value must be initialized");
        }
        
        value.clamp(self.lower_bound, self.upper_bound)
    }

    pub fn sample(&mut self) -> f64 {
        if self.frozen {
            return self.initial_value.expect("Frozen value must be initialized");
        }

        self.uniform_distribution.sample(&mut self.random_generator)
    }

    pub fn get_initial_value(&mut self) -> f64 {
        match self.initial_value {
            None => self.sample(),
            Some(x) => {
                if self.frozen {
                    return x;
                }
                // If not frozen, we might still want a random value based on the initial one,
                // but for now, we just return it. If random variation is needed,
                // this is where the normal_distribution would be used.
                x
            }
        }
    }
}