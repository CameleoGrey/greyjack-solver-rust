
use std::{cmp::Ordering::*, collections::HashMap};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Normal, Distribution, Uniform};
use crate::utils::math_utils;

#[derive(Debug, Clone)]
pub struct GJInteger {
    pub name: String,
    pub initial_value: Option<f64>,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub frozen: bool,
    pub random_generator: StdRng,
    pub uniform_distribution: Uniform<i64>,
    pub normal_distribution: Option<Normal<f64>>,
    pub semantic_groups: Vec<String>
}

impl GJInteger {
    pub fn new(initial_value: Option<i64>, lower_bound: i64, upper_bound: i64, frozen: bool, semantic_groups: Option<Vec<String>>) -> Self {
            
            let normal_distribution;
            match initial_value {
                None => normal_distribution = None,
                Some(x) => normal_distribution = Some(Normal::new(x as f64, 0.1).unwrap())
            };
            
            let casted_initial_value;
            match initial_value {
                Some(x) => casted_initial_value = Some(initial_value.unwrap() as f64),
                None => casted_initial_value = None 
            }

            let mut current_semantic_groups: Vec<String> = Vec::new();
            match semantic_groups {
                None => current_semantic_groups.push("common".to_string()),
                Some(groups) => {
                    for group in groups {
                        current_semantic_groups.push(group);
                    }
                },
            }

            GJInteger {
                name: "".to_string(),
                initial_value: casted_initial_value,
                lower_bound: lower_bound as f64,
                upper_bound: upper_bound as f64,
                frozen: frozen,
                random_generator: StdRng::from_entropy(),
                uniform_distribution: Uniform::new_inclusive(lower_bound, upper_bound),
                normal_distribution: normal_distribution,
                semantic_groups: current_semantic_groups
            }
        }
}

impl GJInteger {

    pub fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    pub fn inverse_transform(&self, value: f64) -> i64 {
        return self.fix(value) as i64;
    }

    pub fn fix(&self, value: f64) -> f64 {

        if self.frozen {
            match self.initial_value {
                Some(x) => return x,
                None => panic!("Frozen value must be initialized")
            }
        }

        let mut fixed_value = Self::min(Self::max(value, self.lower_bound), self.upper_bound);
        fixed_value = math_utils::rint(fixed_value);

        return fixed_value;
    }

    pub fn sample(&mut self) -> f64 {

        if self.frozen {
            match self.initial_value {
                Some(x) => return x,
                None => panic!("Frozen value must be initialized")
            }
        }

        let sampled_value: f64 = self.uniform_distribution.sample( &mut self.random_generator) as f64;
        return sampled_value;
    }

    pub fn get_initial_value(&mut self) -> f64 {

        match self.initial_value {
            None => return self.sample(),
            Some(x) => {
                let initial_value = x;
                if self.frozen {
                    return initial_value as f64;
                }
                return initial_value;
            }

        }

    }

    pub fn min(a: f64, b: f64) -> f64 {

        let min_value;
        match a.total_cmp(&b) {
            Less => min_value = a,
            Greater => min_value = b,
            Equal => min_value = a
        }
        min_value
    }

    pub fn max(a: f64, b: f64) -> f64 {

        let max_value;
        match a.total_cmp(&b) {
            Less => max_value = b,
            Greater => max_value = a,
            Equal => max_value = b
        }
        max_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gp_float_var_frozen() {
        let mut x = GJInteger::new( Some(1), -1, 1, true, None);
        
        let initial_value = x.get_initial_value();
        assert_eq!(initial_value, 1.0);
    }

    #[test]
    fn test_gp_float_var_unfrozen() {
        let mut x = GJInteger::new( Some(1000), -10000, 10000, false, None);
        
        let initial_value = x.get_initial_value();
        assert_ne!(initial_value, 1000.0);
    }

    #[test]
    fn test_gp_float_var_fix_value() {
        let x = GJInteger::new( Some(1), -1, 1, false, None);
        
        let too_little_value: f64 = -100.0;
        let fixed_value = x.fix(too_little_value);
        assert_eq!(fixed_value, -1.0);

        let too_big_value: f64 = 100.0;
        let fixed_value = x.fix(too_big_value);
        assert_eq!(fixed_value, 1.0);
    }

    #[test]
    fn test_gp_float_var_inverse_transform() {
        let x = GJInteger::new( Some(1), -10, 10, false, None);
        
        let x_to_floor = 4.4;
        let x_to_floor = x.inverse_transform(x_to_floor);
        assert_eq!(x_to_floor, 4);

        let x_to_ceil = 4.6;
        let x_to_ceil = x.inverse_transform(x_to_ceil);
        assert_eq!(x_to_ceil, 5);
    }
}