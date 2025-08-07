// src/score_calculation/score_requesters/variables_manager.rs

use crate::{agents::base::Individual, variables::PlanningVariablesVariants};
use crate::variables::PlanningVariablesVariants::*;
use crate::utils::math_utils;
use polars::prelude::*;
use std::collections::HashMap;

use rand::SeedableRng;
use rand::rngs::SmallRng;
use rand_distr::{Distribution, Uniform};

pub struct VariablesManager {
    variables_vec: Vec<PlanningVariablesVariants>,
    pub variables_count: usize,
    pub variable_ids: Vec<usize>,
    pub lower_bounds: Vec<f64>,
    pub upper_bounds: Vec<f64>,

    pub semantic_groups_map: HashMap<String, Vec<usize>>,
    pub semantic_group_keys: Vec<String>,
    pub n_semantic_groups: usize,
    pub discrete_ids: Option<Vec<usize>>
}

impl VariablesManager {
    
    pub fn new(variables_vec: Vec<PlanningVariablesVariants>) -> Self {

        let mut variable_ids: Vec<usize> = Vec::new();
        let mut lower_bounds: Vec<f64> = Vec::new();
        let mut upper_bounds: Vec<f64> = Vec::new();
        let mut discrete_ids: Vec<usize> = Vec::new();

        let variables_count = variables_vec.len();
        for i in 0..variables_count {
            variable_ids.push(i);
            let current_variable = variables_vec.get(i).unwrap();
            match current_variable {
                PlanningVariablesVariants::GJF(x) => {
                    lower_bounds.push(x.lower_bound);
                    upper_bounds.push(x.upper_bound);
                }
                PlanningVariablesVariants::GJI(x) => {
                    lower_bounds.push(x.lower_bound);
                    upper_bounds.push(x.upper_bound);
                    discrete_ids.push(i);
                }
            }
        }

        let semantic_groups_dict = Self::build_semantic_groups_dict(&variables_vec);
        let semantic_group_keys: Vec<String> = semantic_groups_dict.keys().cloned().collect();
        let n_semantic_groups = semantic_group_keys.len();
        let discrete_ids_option = if !discrete_ids.is_empty() {
            Some(discrete_ids)
        } else {
            None
        };

        Self {
            variables_vec,
            variables_count,
            variable_ids,
            lower_bounds,
            upper_bounds,
            semantic_groups_map: semantic_groups_dict,
            semantic_group_keys,
            n_semantic_groups,
            discrete_ids: discrete_ids_option
        }

    }

    fn build_semantic_groups_dict(variables_vec: &Vec<PlanningVariablesVariants>) -> HashMap<String, Vec<usize>> {
        let mut semantic_groups_dict: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, variable) in variables_vec.iter().enumerate() {
            let (variable_semantic_groups, is_frozen_variable) = match variable {
                GJF(x) => (&x.semantic_groups, x.frozen),
                GJI(x) => (&x.semantic_groups, x.frozen),
            };

            for group_name in variable_semantic_groups {
                if !is_frozen_variable {
                    semantic_groups_dict.entry(group_name.clone()).or_default().push(i);
                }
            }
        }
        semantic_groups_dict
    }

    pub fn get_random_semantic_group_ids<'a>(&'a self, rng: &mut SmallRng) -> (&'a Vec<usize>, &'a String) {
        let random_group_id = Uniform::new(0, self.n_semantic_groups).sample(rng);
        let group_name = &self.semantic_group_keys[random_group_id];
        let group_ids = self.semantic_groups_map.get(group_name).unwrap();
        (group_ids, group_name)
    }

    pub fn get_column_random_value(&self, column_id: usize, rng: &mut SmallRng) -> f64 {
        Uniform::new(self.lower_bounds[column_id], self.upper_bounds[column_id]).sample(rng)
    }

    pub fn sample_variables(&mut self) -> Vec<f64> {
        let mut values_array: Vec<f64> = vec![0.0; self.variables_count];
        for i in 0..self.variables_count {
            let variable = &mut self.variables_vec[i];
            let generated_value: f64 = match variable {
                PlanningVariablesVariants::GJF(x) => x.get_initial_value(),
                PlanningVariablesVariants::GJI(x) => x.get_initial_value()
            };
            values_array[i] = generated_value;
        }
        values_array
    }

    pub fn inverse_transform_variables<'a>(&self, values_array: &Vec<f64>) -> Vec<AnyValue<'a>> {
        self.variables_vec.iter().zip(values_array.iter()).map(|(variable, x)| {
            match variable {
                PlanningVariablesVariants::GJF(float_var) => {
                    AnyValue::Float64(float_var.inverse_transform(*x))
                }
                PlanningVariablesVariants::GJI(int_var) => {
                    AnyValue::Int64(int_var.inverse_transform(*x))
                }
            }
        }).collect()
    }

    pub fn inverse_transform_deltas<'a>(&self, deltas: &Vec<Vec<(usize, f64)>>) -> Vec<Vec<(usize, AnyValue<'a>)>> {
        deltas.iter().map(|current_deltas| {
            current_deltas.iter().map(|id_value_tuple| {
                let inverted_value = match &self.variables_vec[id_value_tuple.0] {
                    PlanningVariablesVariants::GJF(float_var) => {
                        AnyValue::Float64(float_var.inverse_transform(id_value_tuple.1))
                    }
                    PlanningVariablesVariants::GJI(int_var) => {
                        AnyValue::Int64(int_var.inverse_transform(id_value_tuple.1))
                    }
                };
                (id_value_tuple.0, inverted_value)
            }).collect()
        }).collect()
    }

    pub fn get_variables_names_vec(&self) -> Vec<String> {
        self.variables_vec.iter().map(|variable| {
            match variable {
                PlanningVariablesVariants::GJF(float_var) => float_var.name.clone(),
                PlanningVariablesVariants::GJI(int_var) => int_var.name.clone()
            }
        }).collect()
    }

    pub fn fix_variables(&self, values_array: &mut Vec<f64>, ids_to_fix: Option<Vec<usize>>) {
        let ids_iter: Box<dyn Iterator<Item = &usize>> = match &ids_to_fix {
            Some(partial_ids) => Box::new(partial_ids.iter()),
            None => Box::new(self.variable_ids.iter())
        };

        for i in ids_iter {
            match &self.variables_vec[*i] {
                GJF(x) => values_array[*i] = x.fix(values_array[*i]),
                GJI(x) => values_array[*i] = x.fix(values_array[*i]),
            }
        }
    }

    pub fn fix_deltas(&self, deltas: &mut Vec<f64>, ids_to_fix: Option<Vec<usize>>) {
        let range_ids = match ids_to_fix {
            Some(partial_ids) => partial_ids,
            None => (0..self.variables_count).collect()
        };

        for (delta_id, &var_id) in range_ids.iter().enumerate() {
            if let Some(delta_val) = deltas.get_mut(delta_id) {
                match &self.variables_vec[var_id] {
                    GJF(x) => *delta_val = x.fix(*delta_val),
                    GJI(x) => *delta_val = x.fix(*delta_val),
                }
            }
        }
    }
}

unsafe impl Send for VariablesManager {}
