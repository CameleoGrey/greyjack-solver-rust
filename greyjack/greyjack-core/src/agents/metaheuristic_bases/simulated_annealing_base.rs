



use std::collections::HashMap;
use crate::score_calculation::score_requesters::VariablesManager;

use super::MetaheuristicBaseTrait;
use crate::score_calculation::scores::ScoreTrait;
use crate::agents::base::Individual;
use std::ops::{AddAssign, Sub};
use std::fmt::Debug;

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rand_distr::{Distribution, Uniform};

use super::Mover;
use super::metaheuristic_kinds_and_names::{MetaheuristicKind, MetaheuristicNames};
use crate::utils::math_utils;
use std::collections::VecDeque;
use std::collections::HashSet;
use std::cmp::max;

/*
https://www.cs.stir.ac.uk/~kjt/techreps/pdf/TR192.pdf
*/

pub struct SimulatedAnnealingBase{

    pub initial_temperature: Vec<f64>,
    pub cooling_rate: Option<f64>,
    pub tabu_entity_rate: f64,

    pub metaheuristic_kind: MetaheuristicKind,
    pub metaheuristic_name: MetaheuristicNames,

    pub group_mutation_rates_map: HashMap<String, f64>,
    pub discrete_ids: Option<Vec<usize>>,
    pub mover: Mover,

    current_temperature: Vec<f64>,
    pub inverted_accomplish_rate: f64,
    random_sampler: Uniform<f64>,
    random_generator: StdRng,
    pub exp: f64,
}

impl SimulatedAnnealingBase  {

    pub fn new(
        initial_temperature: Vec<f64>,
        cooling_rate: Option<f64>,
        tabu_entity_rate: f64,
        mutation_rate_multiplier: Option<f64>,
        move_probas: Option<Vec<f64>>,
        semantic_groups_dict: HashMap<String, Vec<usize>>,
        discrete_ids: Option<Vec<usize>>,
    ) -> Self {

        let current_mutation_rate_multiplier;
        match mutation_rate_multiplier {
            Some(x) => current_mutation_rate_multiplier = mutation_rate_multiplier.unwrap(),
            None => current_mutation_rate_multiplier = 0.0,
        }
        let mut group_mutation_rates_map: HashMap<String, f64> = HashMap::new();
        for group_name in semantic_groups_dict.keys() {
            let group_size = semantic_groups_dict[group_name].len();
            let current_group_mutation_rate = current_mutation_rate_multiplier * (1.0 / (group_size as f64));
            group_mutation_rates_map.insert(group_name.clone(), current_group_mutation_rate);
        }

        Self {
            initial_temperature: initial_temperature.clone(),
            cooling_rate: cooling_rate,
            tabu_entity_rate: tabu_entity_rate,

            metaheuristic_kind: MetaheuristicKind::LocalSearch,
            metaheuristic_name: MetaheuristicNames::SimulatedAnnealing,

            group_mutation_rates_map: group_mutation_rates_map.clone(),
            discrete_ids: discrete_ids.clone(),
            mover: Mover::new(tabu_entity_rate, HashMap::new(), HashMap::new(), HashMap::new(), group_mutation_rates_map.clone(), move_probas),
            current_temperature: initial_temperature,
            inverted_accomplish_rate: 1.0,
            random_sampler: Uniform::new_inclusive(0.0, 1.0),
            random_generator: StdRng::from_entropy(),
            exp: 2.7182818284590452
        }
    }

}

impl<ScoreType> MetaheuristicBaseTrait<ScoreType> for SimulatedAnnealingBase
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

    fn sample_candidates_plain(
            &mut self, 
            population: &mut Vec<Individual<ScoreType>>, 
            current_top_individual: &Individual<ScoreType>,
            variables_manager: &VariablesManager
        ) -> Vec<Vec<f64>> {

        if self.mover.tabu_entity_size_map.len() == 0 {
            let semantic_groups_map = variables_manager.semantic_groups_map.clone();
            for (group_name, group_ids) in semantic_groups_map {
                self.mover.tabu_ids_sets_map.insert(group_name.clone(), HashSet::new());
                self.mover.tabu_entity_size_map.insert(group_name.clone(), max((self.tabu_entity_rate * (group_ids.len() as f64)).ceil() as usize, 1));
                self.mover.tabu_ids_vecdeque_map.insert(group_name.clone(), VecDeque::new());
            }
        }

        let mut candidate = population[0].variable_values.clone();
        let (changed_candidate, changed_columns, candidate_deltas) = self.mover.do_move(&mut candidate, variables_manager, false);
        candidate = changed_candidate.unwrap();
        variables_manager.fix_variables(&mut candidate, changed_columns);
        let candidate = vec![candidate; 1];

        return candidate;

    }

    fn sample_candidates_incremental(
        &mut self,
        population: &mut Vec<Individual<ScoreType>>, 
        current_top_individual: &Individual<ScoreType>,
        variables_manager: &VariablesManager
    ) -> (Vec<f64>, Vec<Vec<(usize, f64)>>) {

        if self.mover.tabu_entity_size_map.len() == 0 {
            let semantic_groups_map = variables_manager.semantic_groups_map.clone();
            for (group_name, group_ids) in semantic_groups_map {
                self.mover.tabu_ids_sets_map.insert(group_name.clone(), HashSet::new());
                self.mover.tabu_entity_size_map.insert(group_name.clone(), max((self.tabu_entity_rate * (group_ids.len() as f64)).ceil() as usize, 1));
                self.mover.tabu_ids_vecdeque_map.insert(group_name.clone(), VecDeque::new());
            }
        }

        let mut candidate = population[0].variable_values.clone();
        let (_, changed_columns, candidate_deltas) = self.mover.do_move(&mut candidate, variables_manager, true);
        let mut candidate_deltas = candidate_deltas.unwrap();
        variables_manager.fix_deltas(&mut candidate_deltas, changed_columns.clone());
        let changed_columns = changed_columns.unwrap();
        let candidate_deltas: Vec<(usize, f64)> = changed_columns.iter().zip(candidate_deltas.iter()).map(|(col_id, delta_value)| (*col_id, *delta_value)).collect();
        let deltas = vec![candidate_deltas; 1];

        return (candidate, deltas);
    }

    fn build_updated_population(
        &mut self, 
        current_population: &Vec<Individual<ScoreType>>, 
        candidates: &mut Vec<Individual<ScoreType>>
        ) -> Vec<Individual<ScoreType>> {
        
        match self.cooling_rate {
            Some(c_r) => self.current_temperature = self.current_temperature.iter().map(|ct| {
                let mut new_temperature = *ct * c_r;
                if new_temperature < 0.000001 {
                    new_temperature = 0.0000001;
                }
                return new_temperature;
            }).collect(),
            None => self.current_temperature = self.current_temperature.iter().map(|ct| self.inverted_accomplish_rate).collect()
        }

        let current_energy = current_population[0].score.as_vec();
        let candidate_energy = candidates[0].score.as_vec();
        let accept_probas: Vec<f64> = current_energy
        .iter().zip(candidate_energy.iter())
        .enumerate()
        .map(|(i, (cur_e, can_e))| self.exp.powf(-((can_e - cur_e) / self.current_temperature[i])))
        .collect();
        
        let accept_proba = accept_probas.iter().fold(1.0, |acc, x| acc * *x);
        let random_value = self.random_sampler.sample(&mut self.random_generator);
        
        let new_population: Vec<Individual<ScoreType>>;
        if (candidates[0].score <= current_population[0].score) || (random_value < accept_proba) {
            new_population = candidates.clone();
        } else {
            new_population = current_population.clone();
        }


        return new_population;
    }

    fn build_updated_population_incremental(
            &mut self, 
            current_population: &Vec<Individual<ScoreType>>, 
            sample: &mut Vec<f64>,
            deltas: Vec<Vec<(usize, f64)>>,
            scores: Vec<ScoreType>,
        ) -> Vec<Individual<ScoreType>> {
        
        match self.cooling_rate {
            Some(c_r) => self.current_temperature = self.current_temperature.iter().map(|ct| {
                let mut new_temperature = *ct * c_r;
                if new_temperature < 0.000001 {
                    new_temperature = 0.0000001;
                }
                return new_temperature;
            }).collect(),
            None => self.current_temperature = self.current_temperature.iter().map(|ct| self.inverted_accomplish_rate).collect()
        }
        

        let current_energy = current_population[0].score.as_vec();
        let candidate_energy = scores[0].as_vec();
        let accept_probas: Vec<f64> = current_energy
        .iter().zip(candidate_energy.iter())
        .enumerate()
        .map(|(i, (cur_e, can_e))| self.exp.powf(-((can_e - cur_e) / self.current_temperature[i])))
        .collect();
        
        let accept_proba = accept_probas.iter().fold(1.0, |acc, x| acc * *x);
        let random_value = self.random_sampler.sample(&mut self.random_generator);
        
        let new_population: Vec<Individual<ScoreType>>;
        if (random_value < accept_proba) {
            let candidate_deltas = &deltas[0];
            for (var_id, new_value) in candidate_deltas {
                sample[*var_id] = *new_value;
            }
            let candidate = Individual::new(sample.clone(), scores[0].clone());
            new_population = vec![candidate; 1];
        } else {
            new_population = current_population.clone();
        }

        return new_population;
    }

    fn get_metaheuristic_kind(&self) -> MetaheuristicKind {
        self.metaheuristic_kind.clone()
    }

    fn get_metaheuristic_name(&self) -> MetaheuristicNames {
        self.metaheuristic_name.clone()
    }
}

unsafe impl Send for SimulatedAnnealingBase {}