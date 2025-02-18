



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

use super::Mover;
use super::metaheuristic_kinds_and_names::{MetaheuristicKind, MetaheuristicNames};
use crate::utils::math_utils;
use std::collections::VecDeque;
use std::collections::HashSet;
use std::cmp::max;

/*
https://www.cs.stir.ac.uk/~kjt/techreps/pdf/TR192.pdf
*/

pub struct LateAcceptanceBase<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq +  PartialOrd + Ord + Debug + Send {

    pub late_acceptance_size: usize,
    //pub late_scores: Vec<ScoreType>,
    pub late_scores: VecDeque<ScoreType>,
    pub tabu_entity_rate: f64,

    pub metaheuristic_kind: MetaheuristicKind,
    pub metaheuristic_name: MetaheuristicNames,

    pub group_mutation_rates_map: HashMap<String, f64>,
    pub discrete_ids: Option<Vec<usize>>,
    pub mover: Mover,
}

impl<ScoreType> LateAcceptanceBase<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq +  PartialOrd + Ord + Debug + Send  {

    pub fn new(
        late_acceptance_size: usize,
        tabu_entity_rate: f64,
        mutation_rate_multiplier: Option<f64>, 
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
            late_acceptance_size: late_acceptance_size,
            tabu_entity_rate: tabu_entity_rate,
            //late_scores: Vec::new(),
            late_scores: VecDeque::new(),


            metaheuristic_kind: MetaheuristicKind::LocalSearch,
            metaheuristic_name: MetaheuristicNames::LateAcceptance,

            group_mutation_rates_map: group_mutation_rates_map.clone(),
            discrete_ids: discrete_ids.clone(),
            mover: Mover::new(tabu_entity_rate, HashMap::new(), HashMap::new(), HashMap::new(), group_mutation_rates_map.clone()),
        }
    }

}

impl<ScoreType> MetaheuristicBaseTrait<ScoreType> for LateAcceptanceBase<ScoreType>
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
    ) -> (Array1<f64>, Vec<Vec<(usize, f64)>>) {

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
        
        let candidate_to_compare_score;
        if self.late_scores.len() == 0 {
            candidate_to_compare_score = current_population[0].score.clone();
        } else {
            // vec variant with sorting
            //self.late_scores.sort();
            //self.late_scores.reverse();
            //candidate_to_compare_score = self.late_scores[0].clone();

            //VecDeque variant
            candidate_to_compare_score = self.late_scores.back().unwrap().clone();
        }

        let mut new_population;
        let candidate_score = candidates[0].score.clone();
        if (candidate_score <= candidate_to_compare_score) || (candidate_score <= current_population[0].score) {
            let best_candidate = candidates[0].clone();
            new_population = vec![best_candidate; 1];

            // vec variant with sorting
            //self.late_scores.push(candidate_score);

            //VecDeque variant
            self.late_scores.push_front(candidate_score);
            if self.late_scores.len() > self.late_acceptance_size {

                // vec variant with sorting
                //self.late_scores = self.late_scores[1..].to_vec();

                // VecDeque variant
                self.late_scores.pop_back();
            }
        } else {
            new_population = current_population.clone();
        }

        return new_population;
    }

    fn build_updated_population_incremental(
            &mut self, 
            current_population: &Vec<Individual<ScoreType>>, 
            sample: &mut Array1<f64>,
            deltas: Vec<Vec<(usize, f64)>>,
            scores: Vec<ScoreType>,
        ) -> Vec<Individual<ScoreType>> {

        let late_native_score;
        if self.late_scores.len() == 0 {
            late_native_score = current_population[0].score.clone();
        } else {
            // vec variant with sorting
            //self.late_scores.sort();
            //self.late_scores.reverse();
            //candidate_to_compare_score = self.late_scores[0].clone();

            //VecDeque variant
            late_native_score = self.late_scores.back().unwrap().clone();
        }

        let candidate_score = scores[0].clone();

        //println!("{:?}", scores);
        let mut new_population:Vec<Individual<ScoreType>>;
        //println!("{:?}, {:?}", candidate_score, late_native_score);
        //println!("{:?}", self.late_scores);
        if (candidate_score <= late_native_score) || (candidate_score <= current_population[0].score) {
            let best_deltas = &deltas[0];
            for (var_id, new_value) in best_deltas {
                sample[*var_id] = *new_value;
            }
            let best_candidate = Individual::new(sample.clone(), candidate_score.clone());
            new_population = vec![best_candidate; 1];

            // vec variant with sorting
            //self.late_scores.push(candidate_score);

            //VecDeque variant
            self.late_scores.push_front(candidate_score);
            if self.late_scores.len() > self.late_acceptance_size {

                // vec variant with sorting
                //self.late_scores = self.late_scores[1..].to_vec();

                // VecDeque variant
                self.late_scores.pop_back();
            }
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

unsafe impl<ScoreType> Send for LateAcceptanceBase<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {}