
use std::collections::{HashMap, HashSet};
use polars::prelude::*;
use crate::core::score_calculation::scores::score_trait::ScoreTrait;
use std::ops::AddAssign;


pub struct OOPScoreCalculator<UtilityObjectVariants, ScoreType>
where 
    ScoreType: ScoreTrait + Clone + AddAssign {
    constraints: HashMap<String, Box<dyn Fn(&mut HashMap<String, DataFrame>, &HashMap<String, DataFrame>) -> Vec<ScoreType>>>,
    constraint_weights: HashMap<String, f64>,
    utility_objects: HashMap<String, UtilityObjectVariants>,
    prescoring_functions: HashMap<String, Box<dyn Fn(&mut HashMap<String, DataFrame>, &HashMap<String, DataFrame>)>>,
}

impl<UtilityObjectVariants, ScoreType> OOPScoreCalculator<UtilityObjectVariants, ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign {

    pub fn new() -> Self {
        Self {
            constraints: HashMap::new(),
            constraint_weights: HashMap::new(),
            utility_objects: HashMap::new(),
            prescoring_functions: HashMap::new(),
        }
    }

    pub fn add_constraint(&mut self, constraint_name: String, constraint_function: Box<dyn Fn(&mut HashMap<String, DataFrame>, &HashMap<String, DataFrame>) -> Vec<ScoreType>>) {
        self.constraints.insert(constraint_name, constraint_function);
    }

    pub fn remove_constraint(&mut self, constraint_name: String) {
        self.constraints.remove(&constraint_name);
    }

    pub fn set_constraint_weights(&mut self, constraint_weigths: HashMap<String, f64>) {
        self.constraint_weights = constraint_weigths;
    }

    pub fn add_utility_object(&mut self, utility_object_name: String, utility_object: UtilityObjectVariants) {
        self.utility_objects.insert(utility_object_name, utility_object);
    }

    pub fn remove_utility_object(&mut self, utility_object_name: String) {
        self.utility_objects.remove(&utility_object_name);
    }

    pub fn add_prescoring_function(&mut self, function_name: String, function: Box<dyn Fn(&mut HashMap<String, DataFrame>, &HashMap<String, DataFrame>)>) {
        self.prescoring_functions.insert(function_name, function);
    }

    pub fn remove_prescoring_function(&mut self, function_name: String) {
        self.prescoring_functions.remove(&function_name);
    }

    fn check_constraint_weights(&mut self) {
        
        // all weights are existing, do nothing
        if self.constraint_weights.keys().len() == self.constraints.len() {
            return;
        }

        // set default constraint weight for existing constraints
        let mut constaint_weight_names: HashSet<String> = HashSet::new();
        let constraint_weights = &mut self.constraint_weights;
        for existing_name in constraint_weights.keys() {
            constaint_weight_names.insert(existing_name.clone());
        }
        for constraint_name in self.constraints.keys() {
            if constaint_weight_names.contains(constraint_name) == false {
                constraint_weights.insert(constraint_name.clone(), 1.0);
            }
        }

    }
    
    pub fn get_score(&mut self, planning_entity_dfs: &mut HashMap<String, DataFrame>, problem_fact_dfs: &HashMap<String, DataFrame>) -> Vec<ScoreType> {

        for prescoring_function_name in self.prescoring_functions.keys() {
            let prescoring_function = self.prescoring_functions.get(prescoring_function_name).unwrap();
            prescoring_function(planning_entity_dfs, problem_fact_dfs);
        }

        self.check_constraint_weights();

        let mut constraint_names: Vec<String> = Vec::new();
        for constraint_name in self.constraints.keys() {
            constraint_names.push(constraint_name.clone());
        }

        let mut scores_vec = Vec::new();
        for constraint_name in constraint_names.clone() {
            let current_constraint_function = self.constraints.get(&constraint_name).unwrap();
            let current_score_vec = current_constraint_function(planning_entity_dfs, problem_fact_dfs);
            scores_vec.push(current_score_vec);
        }
        
        let constraints_count = scores_vec.len();
        let samples_count = scores_vec[0].len();
        let mut scores:Vec<ScoreType> = Vec::new();
        for j in 0..samples_count {
            let mut sample_sum_score = ScoreType::get_null_score();
            for i in 0..constraints_count {
                let constraint_weight = self.constraint_weights[&constraint_names[i]];
                let weighted_score = scores_vec[i][j].mul(constraint_weight);
                sample_sum_score += weighted_score;
            }
            scores.push(sample_sum_score.clone());
        }

        return scores;
        
    }

}