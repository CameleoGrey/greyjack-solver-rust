

use crate::{agents::base::Individual, variables::PlanningVariablesVariants};
use crate::variables::PlanningVariablesVariants::*;
use polars::prelude::*;
use std::collections::HashMap;

use rand::SeedableRng;
use rand::rngs::StdRng;
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
        let semantic_group_keys: Vec<String> = semantic_groups_dict.keys().into_vec().iter().map(|x| x.to_string()).collect();
        let n_semantic_groups = semantic_group_keys.len();
        let discrete_ids_option;
        if discrete_ids.len() != 0 {
            discrete_ids_option = Some(discrete_ids);
        } else {
            discrete_ids_option = None;
        }

        Self {
            variables_vec: variables_vec,
            variables_count: variables_count,
            variable_ids: variable_ids,
            lower_bounds: lower_bounds,
            upper_bounds: upper_bounds,

            semantic_groups_map: semantic_groups_dict,
            semantic_group_keys: semantic_group_keys,
            n_semantic_groups: n_semantic_groups,
            discrete_ids: discrete_ids_option
        }

    }

    fn build_semantic_groups_dict(variables_vec: &Vec<PlanningVariablesVariants>) -> HashMap<String, Vec<usize>> {

        let mut semantic_groups_dict: HashMap<String, Vec<usize>> = HashMap::new();
        for i in 0..variables_vec.len() {
            let variable = &variables_vec[i];
            let variable_semantic_groups;
            let is_frozen_variable;
            match variable {
                GJF(x) => {
                    variable_semantic_groups = &x.semantic_groups;
                    is_frozen_variable = x.frozen;
                },
                GJI(x) => {
                    variable_semantic_groups = &x.semantic_groups;
                    is_frozen_variable = x.frozen;
                },
            }

            for group_name in variable_semantic_groups {
                if semantic_groups_dict.contains_key(group_name) == false {
                    semantic_groups_dict.insert(group_name.clone(), Vec::new());
                }
                if is_frozen_variable {
                    continue;
                }
                semantic_groups_dict.get_mut(group_name).unwrap().push(i);
            }
        }

        return semantic_groups_dict;
    }

    pub fn get_random_semantic_group_ids(&self) -> (&Vec<usize>, &String) {
        let random_group_id = Uniform::new(0, self.n_semantic_groups).sample(&mut StdRng::from_entropy());
        let group_name = &self.semantic_group_keys[random_group_id];
        let group_ids = self.semantic_groups_map.get(group_name).unwrap();
        return (group_ids, group_name);
    }

    pub fn get_column_random_value(&self, column_id: usize) -> f64{
        Uniform::new(self.lower_bounds[column_id], self.upper_bounds[column_id]).sample(&mut StdRng::from_entropy())
    }

    pub fn sample_variables(&mut self) -> Vec<f64> {

        let mut values_array: Vec<f64> = vec![0.0; self.variables_count];
        for i in 0..self.variables_count {

            let variable = &mut self.variables_vec[i];
            let generated_value: f64;
            match variable {
                PlanningVariablesVariants::GJF(x) => generated_value = x.get_initial_value(),
                PlanningVariablesVariants::GJI(x) => generated_value = x.get_initial_value()
            }
            values_array[i] = generated_value;
        }

        return values_array;
    }

    pub fn inverse_transform_variables<'a>(&self, values_array: &Vec<f64>) -> Vec<(AnyValue<'a>)> {


        let values_map: Vec<AnyValue<'a>> =
        self.variables_vec.iter().zip(values_array.iter()).map(|(variable, x)| {
            match variable {
                PlanningVariablesVariants::GJF(float_var) => {
                    AnyValue::Float64(float_var.inverse_transform(*x))
                }
                PlanningVariablesVariants::GJI(int_var) => {
                    AnyValue::Int64(int_var.inverse_transform(*x))
                }
            }
        }).collect();

        return values_map;
    }

    pub fn inverse_transform_deltas<'a>(&self, deltas: &Vec<Vec<(usize, f64)>>) -> Vec<Vec<(usize, AnyValue<'a>)>> {


        let inverted_deltas: Vec<Vec<(usize, AnyValue<'a>)>> =
        deltas.iter().map(|current_deltas| {
            let current_inverted_deltas: Vec<(usize, AnyValue<'a>)> =
            current_deltas.iter().map(|id_value_tuple| {
                let inverted_value;
                match &self.variables_vec[id_value_tuple.0] {
                    PlanningVariablesVariants::GJF(float_var) => {
                        inverted_value = AnyValue::Float64(float_var.inverse_transform(id_value_tuple.1))
                    }
                    PlanningVariablesVariants::GJI(int_var) => {
                        inverted_value = AnyValue::Int64(int_var.inverse_transform(id_value_tuple.1))
                    }
                }
                return (id_value_tuple.0, inverted_value);
            }).collect();
            return current_inverted_deltas;
        }).collect();

        return inverted_deltas;
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

        let range_ids;
        match ids_to_fix {
            Some(partial_ids) => range_ids = partial_ids,
            None => range_ids = Vec::from_iter( (0..self.variables_count).into_iter() )
        }

        let stub_collection: () = range_ids.iter().map(|i| {
            match &self.variables_vec[*i] {
                GJF(x) => values_array[*i] = x.fix(values_array[*i]),
                GJI(x) => values_array[*i] = x.fix(values_array[*i]),
            }
        }).collect();
    }

    pub fn fix_deltas(&self, deltas: &mut Vec<f64>, ids_to_fix: Option<Vec<usize>>) {

        let range_ids;
        match ids_to_fix {
            Some(partial_ids) => range_ids = partial_ids,
            None => range_ids = Vec::from_iter( (0..self.variables_count).into_iter() )
        }

        let _: () = 
        range_ids.iter()
        .enumerate()
        .map(|(delta_id, var_id)| {
            match &self.variables_vec[*var_id] {
                GJF(x) => deltas[delta_id] = x.fix(deltas[delta_id]),
                GJI(x) => deltas[delta_id] = x.fix(deltas[delta_id]),
            }
        }).collect();
    }

}

unsafe impl Send for VariablesManager {}