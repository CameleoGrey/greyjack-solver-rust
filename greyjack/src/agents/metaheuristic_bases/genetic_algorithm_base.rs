

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

use super::moves::Mover;
use super::moves::MoveTrait;
use super::metaheuristic_kinds_and_names::{MetaheuristicKind, MetaheuristicNames};
use crate::utils::math_utils;
use std::collections::VecDeque;
use std::collections::HashSet;
use std::cmp::max;

pub struct GeneticAlgorithmBase {

    pub population_size: usize,
    pub half_population_size: usize,
    pub crossover_probability: f64,
    pub mutation_rate_multiplier: f64,
    pub p_best_rate: f64,
    pub tabu_entity_rate: f64,

    pub metaheuristic_kind: MetaheuristicKind,
    pub metaheuristic_name: MetaheuristicNames,

    pub group_mutation_rates_map: HashMap<String, f64>,
    pub discrete_ids: Option<Vec<usize>>,
    pub mover: Mover,
    pub moves_count: usize,
}

impl GeneticAlgorithmBase {

    pub fn new(
        population_size: usize, 
        crossover_probability: f64,
        p_best_rate: f64,
        tabu_entity_rate: f64,
        mutation_rate_multiplier: Option<f64>, 
        semantic_groups_dict: HashMap<String, Vec<usize>>,
        discrete_ids: Option<Vec<usize>>,
    ) -> Self {

        let half_population_size = (0.5 * (population_size as f64)).ceil() as usize;
        let current_mutation_rate_multiplier;
        match mutation_rate_multiplier {
            Some(x) => current_mutation_rate_multiplier = mutation_rate_multiplier.unwrap(),
            None => current_mutation_rate_multiplier = 0.0 // 0.0 - always use minimal possible move size, 1.0 - is more intuitive,
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
            tabu_entity_rate: tabu_entity_rate,

            metaheuristic_kind: MetaheuristicKind::Population,
            metaheuristic_name: MetaheuristicNames::GeneticAlgorithm,

            group_mutation_rates_map: group_mutation_rates_dict,
            discrete_ids: discrete_ids.clone(),
            mover: Mover::new(tabu_entity_rate, HashMap::new(), HashMap::new(), HashMap::new()),
            moves_count: 6,
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

    fn mutate(&mut self, candidate: &mut Array1<f64>, variables_manager: &VariablesManager, incremental: bool) -> (Option<Array1<f64>>, Option<Vec<usize>>, Option<Vec<f64>>){

        let rand_method_id = Uniform::new(0, self.moves_count).sample(&mut StdRng::from_entropy());
        let changed_candidate: Option<Array1<f64>>;
        let changed_columns: Option<Vec<usize>>;
        let deltas: Option<Vec<f64>>;
        match rand_method_id {
            0 => (changed_candidate, changed_columns, deltas) = self.change_move(candidate, variables_manager, incremental),
            1 => (changed_candidate, changed_columns, deltas) = self.swap_move(candidate, variables_manager, incremental),
            2 => (changed_candidate, changed_columns, deltas) = self.swap_edges_move(candidate, variables_manager, incremental),
            3 => (changed_candidate, changed_columns, deltas) = self.scramble_move(candidate, variables_manager, incremental),
            4 => (changed_candidate, changed_columns, deltas) = self.insertion_move(candidate, variables_manager, incremental),
            5 => (changed_candidate, changed_columns, deltas) = self.inverse_move(candidate, variables_manager, incremental),
            _ => panic!("Invalid rand_method_id, no move with such id"),
        }

        return (changed_candidate, changed_columns, deltas);
    }

}

impl<ScoreType> MetaheuristicBaseTrait<ScoreType> for GeneticAlgorithmBase
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

    fn sample_candidates_plain(
            &mut self, 
            population: &mut Vec<Individual<ScoreType>>, 
            current_top_individual: &Individual<ScoreType>,
            variables_manager: &VariablesManager
        ) -> Vec<Array1<f64>> {

        if self.mover.tabu_entity_size_map.len() == 0 {
            let semantic_groups_map = variables_manager.semantic_groups_map.clone();
            for (group_name, group_ids) in semantic_groups_map {
                self.mover.tabu_ids_sets_map.insert(group_name.clone(), HashSet::new());
                self.mover.tabu_entity_size_map.insert(group_name.clone(), max((self.tabu_entity_rate * (group_ids.len() as f64)).ceil() as usize, 1));
                self.mover.tabu_ids_vecdeque_map.insert(group_name.clone(), VecDeque::new());
            }
        }
        
        population.sort();

        let mut candidates: Vec<Array1<f64>> = Vec::new();
        for i in 0..self.half_population_size {
            let mut candidate_1 = self.select_p_best(population).variable_values;
            let mut candidate_2 = self.select_p_best(population).variable_values;

            if Uniform::new_inclusive(0.0, 1.0).sample(&mut StdRng::from_entropy()) <= self.crossover_probability {
                (candidate_1, candidate_2) = self.cross(candidate_1, candidate_2);
            }
            
            let (mut changed_candidate_1, changed_columns_1, candidate_deltas_1) = self.mutate(&mut candidate_1, variables_manager, false);
            let (mut changed_candidate_2, changed_columns_2, candidate_deltas_2) = self.mutate(&mut candidate_2, variables_manager, false);

            candidate_1 = changed_candidate_1.unwrap();
            candidate_2 = changed_candidate_2.unwrap();


            // for crossover with rint() one doesn't need for fixing the whole candidate vector
            // float values are crossed without rint, but due to the convex sum they will be still into the bounds
            // all sampled values are always in the bounds
            // problems can occur only by swap mutations, so fix all changed by a move columns
            variables_manager.fix_variables(&mut candidate_1, changed_columns_1);
            variables_manager.fix_variables(&mut candidate_2, changed_columns_2);

            candidates.push(candidate_1);
            candidates.push(candidate_2);
        }
        
        return candidates;
    }

    fn sample_candidates_incremental(
        &mut self,
        population: &mut Vec<Individual<ScoreType>>, 
        current_top_individual: &Individual<ScoreType>,
        variables_manager: &VariablesManager
    ) -> (Array1<f64>, Vec<Vec<(usize, f64)>>) {
        panic!("Incremental candidates sampling is available only for local search approaches (TabuSearch, LateAcceptance, etc).")
    }

    fn build_updated_population(
        &mut self, 
        current_population: &Vec<Individual<ScoreType>>, 
        candidates: &mut Vec<Individual<ScoreType>>
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

    fn build_updated_population_incremental(
            &mut self, 
            current_population: &Vec<Individual<ScoreType>>, 
            sample: &mut Array1<f64>,
            deltas: Vec<Vec<(usize, f64)>>,
            scores: Vec<ScoreType>,
        ) -> Vec<Individual<ScoreType>> {
        
        panic!("Incremental candidates sampling is available only for local search approaches (TabuSearch, LateAcceptance, etc).")
    }

    fn get_metaheuristic_kind(&self) -> MetaheuristicKind {
        self.metaheuristic_kind.clone()
    }

    fn get_metaheuristic_name(&self) -> MetaheuristicNames {
        self.metaheuristic_name.clone()
    }
}

impl MoveTrait for GeneticAlgorithmBase {

    fn get_necessary_info_for_move<'d>(
        &self,
        variables_manager: &'d VariablesManager
    ) -> (&'d Vec<usize>, &'d String, usize) {
        
        let (group_ids, group_name) = variables_manager.get_random_semantic_group_ids();
        let group_mutation_rate = self.group_mutation_rates_map[group_name];
        let random_values = Array1::random(variables_manager.variables_count, Uniform::new_inclusive(0.0, 1.0));
        let crossover_mask: Array1<bool> = random_values.iter().map(|x| x < &group_mutation_rate).collect();
        let mut current_change_count = crossover_mask.iter().filter(|x| **x == true).count();

        return (group_ids, group_name, current_change_count);
    }

    fn change_move(
        &mut self, 
        candidate: &Array1<f64>, 
        variables_manager: &VariablesManager,
        incremental: bool,
    ) -> (Option<Array1<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {

        let (group_ids, group_name, current_change_count) = self.get_necessary_info_for_move(variables_manager);
        self.mover.change_move_base(candidate, variables_manager, current_change_count, &group_ids, group_name, incremental)

    }

    fn swap_move(
        &mut self, 
        candidate: &Array1<f64>, 
        variables_manager: &VariablesManager, 
        incremental: bool,
    ) -> (Option<Array1<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {

            let (group_ids, group_name, current_change_count) = self.get_necessary_info_for_move(variables_manager);
        
            self.mover.swap_move_base(candidate, variables_manager, current_change_count, &group_ids, group_name, incremental)
    }
    
    fn swap_edges_move(
        &mut self, 
        candidate: &Array1<f64>, 
        variables_manager: &VariablesManager, 
        incremental: bool,
    ) -> (Option<Array1<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {

        let (group_ids, group_name, current_change_count) = self.get_necessary_info_for_move(variables_manager);
        
        self.mover.swap_edges_move_base(candidate, variables_manager, current_change_count, &group_ids, group_name, incremental)
    }

    fn scramble_move(
        &mut self, 
        candidate: &Array1<f64>, 
        variables_manager: &VariablesManager, 
        incremental: bool,
    ) -> (Option<Array1<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {
        
        let mut current_change_count = Uniform::new_inclusive(3, 6).sample(&mut StdRng::from_entropy());
        let (group_ids, group_name) = variables_manager.get_random_semantic_group_ids();

        self.mover.scramble_move_base(candidate, variables_manager, current_change_count, &group_ids, group_name, incremental)
    }

    fn insertion_move(
        &mut self, 
        candidate: &Array1<f64>, 
        variables_manager: &VariablesManager, 
        incremental: bool,
    ) -> (Option<Array1<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {

        let (group_ids, group_name) = variables_manager.get_random_semantic_group_ids();
        let current_change_count = 2;

        self.mover.insertion_move_base(candidate, variables_manager, current_change_count, &group_ids, group_name, incremental)
    
    }

    fn inverse_move(
        &mut self, 
        candidate: &Array1<f64>, 
        variables_manager: &VariablesManager,
        incremental: bool,
    ) -> (Option<Array1<f64>>, Option<Vec<usize>>, Option<Vec<f64>>) {

        let (group_ids, group_name) = variables_manager.get_random_semantic_group_ids();
        let current_change_count = 2;

        self.mover.inverse_move_base(candidate, variables_manager, current_change_count, &group_ids, group_name, incremental)

    }
}

unsafe impl Send for GeneticAlgorithmBase {}