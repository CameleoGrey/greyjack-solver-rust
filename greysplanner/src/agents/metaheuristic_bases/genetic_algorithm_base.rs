

use std::collections::HashMap;
use crate::score_calculation::score_requesters::VariablesManager;

use super::MetaheuristicBaseTrait;
use crate::score_calculation::scores::ScoreTrait;
use crate::agents::base::Individual;
use ndarray::Array1;
use ndarray_rand::RandomExt;
use std::ops::AddAssign;
use std::fmt::Debug;

use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Uniform};

use super::moves::BaseMoves;
use super::moves::MoveTrait;
use super::metaheuristic_kinds_and_names::{MetaheuristicKind, MetaheuristicNames};
use crate::utils::math_utils;

pub struct GeneticAlgorithmBase {

    pub population_size: usize,
    pub half_population_size: usize,
    pub crossover_probability: f64,
    pub mutation_rate_multiplier: f64,
    pub p_best_rate: f64,

    pub metaheuristic_kind: MetaheuristicKind,
    pub metaheuristic_name: MetaheuristicNames,

    pub group_mutation_rates_dict: HashMap<String, f64>,
    pub discrete_ids: Option<Vec<usize>>,
    pub moves_count: usize,
}

impl GeneticAlgorithmBase {

    pub fn new(
        population_size: usize, 
        crossover_probability: f64, 
        mutation_rate_multiplier: Option<f64>, 
        p_best_rate: f64,
        semantic_groups_dict: HashMap<String, Vec<usize>>,
        discrete_ids: Option<Vec<usize>>,
    ) -> Self {

        let half_population_size = (0.5 * (population_size as f64)).ceil() as usize;
        let current_mutation_rate_multiplier;
        match mutation_rate_multiplier {
            Some(x) => current_mutation_rate_multiplier = mutation_rate_multiplier.unwrap(),
            None => current_mutation_rate_multiplier = 1.0,
        }
        let mut group_mutation_rates_dict: HashMap<String, f64> = HashMap::new();
        for group_name in semantic_groups_dict.keys() {
            let group_size = semantic_groups_dict[group_name].len();
            let current_group_mutation_rate = current_mutation_rate_multiplier * (1.0 / (group_size as f64));
            group_mutation_rates_dict.insert(group_name.clone(), current_group_mutation_rate);
        }

        Self {
            population_size: population_size,
            half_population_size: half_population_size,
            crossover_probability: crossover_probability,
            mutation_rate_multiplier: current_mutation_rate_multiplier,
            p_best_rate: p_best_rate,

            metaheuristic_kind: MetaheuristicKind::Population,
            metaheuristic_name: MetaheuristicNames::GeneticAlgorithm,

            group_mutation_rates_dict: group_mutation_rates_dict,
            discrete_ids: discrete_ids.clone(),
            moves_count: 5,
        }
    }

    fn select_p_best<ScoreType>(&mut self, population: &Vec<Individual<ScoreType>>) -> Individual<ScoreType>
    where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

        let p_best_proba = Uniform::new(0.000001, self.p_best_rate).sample(&mut StdRng::from_entropy());
        let last_top_id = (p_best_proba * (self.population_size as f64)).ceil() as usize;
        let chosen_id:usize = Uniform::new(0, last_top_id).sample(&mut StdRng::from_entropy());
        let p_best = population[chosen_id].clone();

        return p_best;
    }

    fn select_p_worst<ScoreType>(&mut self, population: &Vec<Individual<ScoreType>>) -> Individual<ScoreType>
    where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

        let p_best_proba = Uniform::new(0.000001, self.p_best_rate).sample(&mut StdRng::from_entropy());
        let last_top_id = (p_best_proba * (self.population_size as f64)).ceil() as usize;
        let chosen_id: usize = Uniform::new(self.population_size - last_top_id, self.population_size).sample(&mut StdRng::from_entropy());
        let p_worst = population[chosen_id].clone();

        return p_worst;
    }

    fn cross(&mut self, candidate_1: Array1<f64>, candidate_2: Array1<f64>) -> (Array1<f64>, Array1<f64>) {

        let variables_count = candidate_1.len();
        let mut weights = Array1::random(variables_count, Uniform::new_inclusive(0.0, 1.0));

        match &self.discrete_ids {
            None => (),
            Some(discrete_ids) => discrete_ids.into_iter().for_each(|i| weights[*i] = math_utils::rint(weights[*i]))
        }

        let new_candidate_1: Array1<f64> = 
            weights.iter()
            .zip(candidate_1.iter())
            .zip(candidate_2.iter())
            .map(|((w, c_1), c_2)| {
                c_1 * w + c_2 * (1.0 - w)
            })
            .collect();

        let new_candidate_2: Array1<f64> = 
            weights.iter()
            .zip(candidate_1.iter())
            .zip(candidate_2.iter())
            .map(|((w, c_1), c_2)| {
                c_2 * w + c_1 * (1.0 - w)
            })
            .collect();

        return (new_candidate_1, new_candidate_2);
    }

    fn mutate(&mut self, candidate: &mut Array1<f64>, variables_manager: &VariablesManager) -> Option<Vec<usize>>{

        let rand_method_id = Uniform::new(0, self.moves_count).sample(&mut StdRng::from_entropy());
        let changed_columns: Option<Vec<usize>>;
        match rand_method_id {
            0 => changed_columns = Self::change_move(candidate, variables_manager, &self.group_mutation_rates_dict, variables_manager.variables_count),
            1 => changed_columns = Self::swap_move(candidate, variables_manager, &self.group_mutation_rates_dict, variables_manager.variables_count),
            2 => changed_columns = Self::swap_edges_move(candidate, variables_manager, &self.group_mutation_rates_dict, variables_manager.variables_count),
            3 => changed_columns = Self::insertion_move(candidate, variables_manager, &self.group_mutation_rates_dict, variables_manager.variables_count),
            4 => changed_columns = Self::scramble_move(candidate, variables_manager, &self.group_mutation_rates_dict, variables_manager.variables_count),
            _ => panic!("Invalid rand_method_id, no move with such id"),
        }

        return changed_columns;
    }

}

impl<ScoreType> MetaheuristicBaseTrait<ScoreType> for GeneticAlgorithmBase
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

    fn sample_candidates(
            &mut self, 
            population: &mut Vec<Individual<ScoreType>>, 
            current_top_individual: &Individual<ScoreType>,
            variables_manager: &VariablesManager
        ) -> Vec<Array1<f64>> {
        
        population.sort();

        let mut candidates: Vec<Array1<f64>> = Vec::new();
        for i in 0..self.half_population_size {
            let mut candidate_1 = self.select_p_best(population).variable_values;
            let mut candidate_2 = self.select_p_best(population).variable_values;

            if Uniform::new_inclusive(0.0, 1.0).sample(&mut StdRng::from_entropy()) <= self.crossover_probability {
                (candidate_1, candidate_2) = self.cross(candidate_1, candidate_2);
            }
            
            let candidate_1_changed_columns = self.mutate(&mut candidate_1, variables_manager);
            let candidate_2_changed_columns = self.mutate(&mut candidate_2, variables_manager);

            // for crossover with np.rint() one doesn't need for fixing the whole candidate vector
            // float values are crossed without rint, but due to the convex sum they will be still into the bounds
            // all sampled values are always in the bounds
            // problems can occur only by swap mutations, so fix all changed by a move columns
            variables_manager.fix_variables(&mut candidate_1, candidate_1_changed_columns);
            variables_manager.fix_variables(&mut candidate_2, candidate_2_changed_columns);

            candidates.push(candidate_1);
            candidates.push(candidate_2);
        }
        
        return candidates;
    }

    fn build_updated_population(
        &mut self, 
        current_population: &Vec<Individual<ScoreType>>, 
        candidates: &Vec<Individual<ScoreType>>
        ) -> Vec<Individual<ScoreType>> {
        
        let mut winners: Vec<Individual<ScoreType>> = Vec::new();
        for i in 0..self.population_size {
            let weak_native = self.select_p_worst(current_population);
            let candidate = &candidates[i];
            let winner = if &candidate.score <= &weak_native.score {candidate.clone()} else {weak_native.clone()};
            winners.push(winner);
        }

        return winners;
    }

    fn get_metaheuristic_kind(&self) -> MetaheuristicKind {
        self.metaheuristic_kind.clone()
    }

    fn get_metaheuristic_name(&self) -> MetaheuristicNames {
        self.metaheuristic_name.clone()
    }
}

impl MoveTrait for GeneticAlgorithmBase {

    fn get_needful_info_for_move<'d>(
            variables_manager: &'d VariablesManager, 
            group_mutation_rates_dict: &HashMap<String, f64>, 
            variables_count: usize
        ) -> (&'d Vec<usize>, usize) {
        
            let (group_ids, group_name) = variables_manager.get_random_semantic_group_ids();
            let group_mutation_rate = group_mutation_rates_dict[group_name];
            let random_values = Array1::random(variables_count, Uniform::new_inclusive(0.0, 1.0));
            let crossover_mask: Array1<bool> = random_values.iter().map(|x| x < &group_mutation_rate).collect();
            let mut current_change_count = crossover_mask.iter().filter(|x| **x == true).count();

            return (group_ids, current_change_count);
    }

    fn change_move(
            candidate: &mut Array1<f64>, 
            variables_manager: &VariablesManager, 
            group_mutation_rates_dict: &HashMap<String, f64>, 
            variables_count: usize
        ) -> Option<Vec<usize>> {

            let (group_ids, current_change_count) = Self::get_needful_info_for_move(variables_manager, group_mutation_rates_dict, variables_count);

            BaseMoves::change_move_base(candidate, variables_manager, current_change_count, &group_ids)   
    }

    fn swap_move(
            candidate: &mut Array1<f64>, 
            variables_manager: &VariablesManager, 
            group_mutation_rates_dict: &HashMap<String, f64>, 
            variables_count: usize
        ) -> Option<Vec<usize>> {

            let (group_ids, current_change_count) = Self::get_needful_info_for_move(variables_manager, group_mutation_rates_dict, variables_count);
        
            BaseMoves::swap_move_base(candidate, variables_manager, current_change_count, &group_ids)
    }
    
    fn swap_edges_move(
            candidate: &mut Array1<f64>, 
            variables_manager: &VariablesManager, 
            group_mutation_rates_dict: &HashMap<String, f64>, 
            variables_count: usize
        ) -> Option<Vec<usize>> {

            let (group_ids, current_change_count) = Self::get_needful_info_for_move(variables_manager, group_mutation_rates_dict, variables_count);
            
            BaseMoves::swap_edges_move_base(candidate, variables_manager, current_change_count, &group_ids)
    }

    fn insertion_move(
            candidate: &mut Array1<f64>, 
            variables_manager: &VariablesManager, 
            group_mutation_rates_dict: &HashMap<String, f64>, 
            variables_count: usize
        ) -> Option<Vec<usize>> {

            let (group_ids, group_name) = variables_manager.get_random_semantic_group_ids();
            let current_change_count = 2;

            BaseMoves::insertion_move_base(candidate, variables_manager, current_change_count, group_ids)
        
    }

    fn scramble_move(
            candidate: &mut Array1<f64>, 
            variables_manager: &VariablesManager, 
            group_mutation_rates_dict: 
            &HashMap<String, f64>, variables_count: usize
        ) -> Option<Vec<usize>> {
        
            let mut current_change_count = Uniform::new_inclusive(3, 6).sample(&mut StdRng::from_entropy());
            let (group_ids, group_name) = variables_manager.get_random_semantic_group_ids();

            BaseMoves::scramble_move_base(candidate, variables_manager, current_change_count, group_ids)
    }
}

unsafe impl Send for GeneticAlgorithmBase {}