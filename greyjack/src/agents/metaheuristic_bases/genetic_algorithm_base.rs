

use std::collections::HashMap;
use crate::score_calculation::score_requesters::VariablesManager;

use super::MetaheuristicBaseTrait;
use crate::score_calculation::scores::ScoreTrait;
use crate::agents::base::Individual;
use std::ops::AddAssign;
use std::fmt::Debug;

use rand::SeedableRng;
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
        let current_mutation_rate_multiplier;
        match mutation_rate_multiplier {
            Some(x) => current_mutation_rate_multiplier = mutation_rate_multiplier.unwrap(),
            None => current_mutation_rate_multiplier = 0.0 // 0.0 - always use minimal possible move size, 1.0 - is more intuitive,
        }
        let mut group_mutation_rates_map: HashMap<String, f64> = HashMap::new();
        for group_name in semantic_groups_dict.keys() {
            let group_size = semantic_groups_dict[group_name].len();
            let current_group_mutation_rate = current_mutation_rate_multiplier * (1.0 / (group_size as f64));
            group_mutation_rates_map.insert(group_name.clone(), current_group_mutation_rate);
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

            group_mutation_rates_map: group_mutation_rates_map.clone(),
            discrete_ids: discrete_ids.clone(),
            mover: Mover::new(tabu_entity_rate, HashMap::new(), HashMap::new(), HashMap::new(), group_mutation_rates_map.clone(), move_probas),
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

    fn cross(&mut self, candidate_1: Vec<f64>, candidate_2: Vec<f64>) -> (Vec<f64>, Vec<f64>) {

        let variables_count = candidate_1.len();
        let mut weights = vec![Uniform::new_inclusive(0.0, 1.0).sample(&mut StdRng::from_entropy()); variables_count];

        match &self.discrete_ids {
            None => (),
            Some(discrete_ids) => discrete_ids.into_iter().for_each(|i| weights[*i] = math_utils::rint(weights[*i]))
        }

        let new_candidate_1: Vec<f64> = 
            weights.iter()
            .zip(candidate_1.iter())
            .zip(candidate_2.iter())
            .map(|((w, c_1), c_2)| {
                c_1 * w + c_2 * (1.0 - w)
            })
            .collect();

        let new_candidate_2: Vec<f64> = 
            weights.iter()
            .zip(candidate_1.iter())
            .zip(candidate_2.iter())
            .map(|((w, c_1), c_2)| {
                c_2 * w + c_1 * (1.0 - w)
            })
            .collect();

        return (new_candidate_1, new_candidate_2);
    }

}

impl<ScoreType> MetaheuristicBaseTrait<ScoreType> for GeneticAlgorithmBase
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
        
        population.sort();

        let mut candidates: Vec<Vec<f64>> = Vec::new();
        for i in 0..self.half_population_size {
            let mut candidate_1 = self.select_p_best(population).variable_values;
            let mut candidate_2 = self.select_p_best(population).variable_values;

            if Uniform::new_inclusive(0.0, 1.0).sample(&mut StdRng::from_entropy()) <= self.crossover_probability {
                (candidate_1, candidate_2) = self.cross(candidate_1, candidate_2);
            }
            
            let (mut changed_candidate_1, changed_columns_1, candidate_deltas_1) = self.mover.do_move(&mut candidate_1, variables_manager, false);
            let (mut changed_candidate_2, changed_columns_2, candidate_deltas_2) = self.mover.do_move(&mut candidate_2, variables_manager, false);

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
    ) -> (Vec<f64>, Vec<Vec<(usize, f64)>>) {
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
            sample: &mut Vec<f64>,
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

unsafe impl Send for GeneticAlgorithmBase {}