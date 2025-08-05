// src/agents/metaheuristic_bases/genetic_algorithm_base.rs

use std::collections::HashMap;
use crate::score_calculation::score_requesters::VariablesManager;
use super::MetaheuristicBaseTrait;
use crate::score_calculation::scores::ScoreTrait;
use crate::agents::base::Individual;
use std::ops::{AddAssign, Sub};
use std::fmt::Debug;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Uniform};
use super::Mover;
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
    rng: StdRng,
}

impl GeneticAlgorithmBase {
    pub fn new(
        population_size: usize,
        crossover_probability: f64,
        p_best_rate: f64,
        tabu_entity_rate: f64,
        mutation_rate_multiplier: Option<f64>,
        move_probas: Option<Vec<f64>>,
        semantic_groups_dict: HashMap<String, Vec<usize>>,
        discrete_ids: Option<Vec<usize>>,
    ) -> Self {
        let half_population_size = (0.5 * (population_size as f64)).ceil() as usize;
        let current_mutation_rate_multiplier = mutation_rate_multiplier.unwrap_or(0.0);
        
        let mut group_mutation_rates_map: HashMap<String, f64> = HashMap::new();
        for (group_name, group_ids) in &semantic_groups_dict {
            let group_size = group_ids.len();
            let current_group_mutation_rate = current_mutation_rate_multiplier * (1.0 / (group_size as f64));
            group_mutation_rates_map.insert(group_name.clone(), current_group_mutation_rate);
        }

        Self {
            population_size,
            half_population_size,
            crossover_probability,
            mutation_rate_multiplier: current_mutation_rate_multiplier,
            p_best_rate,
            tabu_entity_rate,
            metaheuristic_kind: MetaheuristicKind::Population,
            metaheuristic_name: MetaheuristicNames::GeneticAlgorithm,
            group_mutation_rates_map: group_mutation_rates_map.clone(),
            discrete_ids: discrete_ids.clone(),
            mover: Mover::new(tabu_entity_rate, HashMap::new(), HashMap::new(), HashMap::new(), group_mutation_rates_map, move_probas),
            rng: math_utils::create_rng(),
        }
    }

    fn select_p_best<ScoreType>(&mut self, population: &Vec<Individual<ScoreType>>) -> Individual<ScoreType>
    where
        ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send,
    {
        let p_best_proba = Uniform::new(0.000001, self.p_best_rate).sample(&mut self.rng);
        let last_top_id = (p_best_proba * (self.population_size as f64)).ceil() as usize;
        let chosen_id: usize = Uniform::new(0, last_top_id.max(1)).sample(&mut self.rng);
        population[chosen_id].clone()
    }

    fn select_p_worst<ScoreType>(&mut self, population: &Vec<Individual<ScoreType>>) -> Individual<ScoreType>
    where
        ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send,
    {
        let p_best_proba = Uniform::new(0.000001, self.p_best_rate).sample(&mut self.rng);
        let last_top_id = (p_best_proba * (self.population_size as f64)).ceil() as usize;
        let chosen_id: usize = Uniform::new(self.population_size.saturating_sub(last_top_id), self.population_size).sample(&mut self.rng);
        population[chosen_id].clone()
    }

    fn cross(&mut self, candidate_1: Vec<f64>, candidate_2: Vec<f64>) -> (Vec<f64>, Vec<f64>) {
        let variables_count = candidate_1.len();
        let mut weights: Vec<f64> = (0..variables_count)
            .map(|_| Uniform::new_inclusive(0.0, 1.0).sample(&mut self.rng))
            .collect();

        if let Some(discrete_ids) = &self.discrete_ids {
            for &i in discrete_ids {
                weights[i] = math_utils::rint(weights[i]);
            }
        }

        let new_candidate_1: Vec<f64> = weights.iter()
            .zip(candidate_1.iter())
            .zip(candidate_2.iter())
            .map(|((w, c_1), c_2)| c_1 * w + c_2 * (1.0 - w))
            .collect();

        let new_candidate_2: Vec<f64> = weights.iter()
            .zip(candidate_1.iter())
            .zip(candidate_2.iter())
            .map(|((w, c_1), c_2)| c_2 * w + c_1 * (1.0 - w))
            .collect();

        (new_candidate_1, new_candidate_2)
    }
}

impl<ScoreType> MetaheuristicBaseTrait<ScoreType> for GeneticAlgorithmBase
where
    ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send,
{
    fn sample_candidates_plain(
        &mut self,
        population: &mut Vec<Individual<ScoreType>>,
        _current_top_individual: &Individual<ScoreType>,
        variables_manager: &VariablesManager,
    ) -> Vec<Vec<f64>> {
        if self.mover.tabu_entity_size_map.is_empty() {
            let semantic_groups_map = variables_manager.semantic_groups_map.clone();
            for (group_name, group_ids) in semantic_groups_map {
                self.mover.tabu_ids_sets_map.insert(group_name.clone(), HashSet::new());
                self.mover.tabu_entity_size_map.insert(group_name.clone(), max((self.tabu_entity_rate * (group_ids.len() as f64)).ceil() as usize, 1));
                self.mover.tabu_ids_vecdeque_map.insert(group_name.clone(), VecDeque::new());
            }
        }

        population.sort();

        let mut candidates: Vec<Vec<f64>> = Vec::with_capacity(self.population_size);
        for _ in 0..self.half_population_size {
            let mut candidate_1 = self.select_p_best(population).variable_values;
            let mut candidate_2 = self.select_p_best(population).variable_values;

            if Uniform::new_inclusive(0.0, 1.0).sample(&mut self.rng) <= self.crossover_probability {
                (candidate_1, candidate_2) = self.cross(candidate_1, candidate_2);
            }

            let (changed_candidate_1, changed_columns_1, _) = self.mover.do_move(&candidate_1, variables_manager, false);
            let (changed_candidate_2, changed_columns_2, _) = self.mover.do_move(&candidate_2, variables_manager, false);

            let mut final_candidate_1 = changed_candidate_1.unwrap_or(candidate_1);
            let mut final_candidate_2 = changed_candidate_2.unwrap_or(candidate_2);

            variables_manager.fix_variables(&mut final_candidate_1, changed_columns_1);
            variables_manager.fix_variables(&mut final_candidate_2, changed_columns_2);

            candidates.push(final_candidate_1);
            if candidates.len() < self.population_size {
                candidates.push(final_candidate_2);
            }
        }
        candidates
    }

    fn sample_candidates_incremental(
        &mut self,
        _population: &mut Vec<Individual<ScoreType>>,
        _current_top_individual: &Individual<ScoreType>,
        _variables_manager: &VariablesManager,
    ) -> (Vec<f64>, Vec<Vec<(usize, f64)>>) {
        panic!("Incremental candidates sampling is available only for local search approaches (TabuSearch, LateAcceptance, etc).");
    }

    fn build_updated_population(
        &mut self,
        current_population: &Vec<Individual<ScoreType>>,
        candidates: &mut Vec<Individual<ScoreType>>,
    ) -> Vec<Individual<ScoreType>> {
        let mut winners: Vec<Individual<ScoreType>> = Vec::with_capacity(self.population_size);
        for i in 0..self.population_size {
            let weak_native = self.select_p_worst(current_population);
            let candidate = &candidates[i];
            let winner = if candidate.score <= weak_native.score {
                candidate.clone()
            } else {
                weak_native
            };
            winners.push(winner);
        }
        winners
    }

    fn build_updated_population_incremental(
        &mut self,
        _current_population: &Vec<Individual<ScoreType>>,
        _sample: &mut Vec<f64>,
        _deltas: Vec<Vec<(usize, f64)>>,
        _scores: Vec<ScoreType>,
    ) -> (Vec<Individual<ScoreType>>, Option<Vec<(usize, f64)>>) {
        panic!("Incremental candidates sampling is available only for local search approaches (TabuSearch, LateAcceptance, etc).");
    }

    fn get_metaheuristic_kind(&self) -> MetaheuristicKind {
        self.metaheuristic_kind.clone()
    }

    fn get_metaheuristic_name(&self) -> MetaheuristicNames {
        self.metaheuristic_name.clone()
    }
}

unsafe impl Send for GeneticAlgorithmBase {}
