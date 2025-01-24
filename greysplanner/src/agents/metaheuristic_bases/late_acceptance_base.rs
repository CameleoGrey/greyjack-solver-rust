



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
use std::collections::VecDeque;

/*
https://www.cs.stir.ac.uk/~kjt/techreps/pdf/TR192.pdf
*/

pub struct LateAcceptanceBase<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq +  PartialOrd + Ord + Debug + Send {

    pub population_size: usize,
    pub late_acceptance_size: usize,
    //pub late_scores: Vec<ScoreType>,
    pub late_scores: VecDeque<ScoreType>,

    pub metaheuristic_kind: MetaheuristicKind,
    pub metaheuristic_name: MetaheuristicNames,

    pub group_mutation_rates_dict: HashMap<String, f64>,
    pub discrete_ids: Option<Vec<usize>>,
    pub moves_count: usize,
}

impl<ScoreType> LateAcceptanceBase<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq +  PartialOrd + Ord + Debug + Send  {

    pub fn new(
        population_size: usize,
        late_acceptance_size: usize,
        mutation_rate_multiplier: Option<f64>, 
        semantic_groups_dict: HashMap<String, Vec<usize>>,
        discrete_ids: Option<Vec<usize>>,
    ) -> Self {

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
            late_acceptance_size: late_acceptance_size,
            //late_scores: Vec::new(),
            late_scores: VecDeque::new(),

            metaheuristic_kind: MetaheuristicKind::LocalSearch,
            metaheuristic_name: MetaheuristicNames::LateAcceptance,

            group_mutation_rates_dict: group_mutation_rates_dict,
            discrete_ids: discrete_ids.clone(),
            moves_count: 5,
        }
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

impl<ScoreType> MetaheuristicBaseTrait<ScoreType> for LateAcceptanceBase<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

    fn sample_candidates(
            &mut self, 
            population: &mut Vec<Individual<ScoreType>>, 
            current_top_individual: &Individual<ScoreType>,
            variables_manager: &VariablesManager
        ) -> Vec<Array1<f64>> {
        
        if population.len() > 1 {
            population.sort();
        }

        let mut candidate = population[0].variable_values.clone();
        let changed_columns = self.mutate(&mut candidate, variables_manager);
        variables_manager.fix_variables(&mut candidate, changed_columns);
        let candidate = vec![candidate; 1];

        return candidate;

    }

    fn build_updated_population(
        &mut self, 
        current_population: &Vec<Individual<ScoreType>>, 
        candidates: &Vec<Individual<ScoreType>>
        ) -> (Vec<Individual<ScoreType>>, bool) {
        
        let candidate_to_compare_score;
        if self.late_scores.len() == 0 {
            candidate_to_compare_score = current_population[current_population.len() - 1].score.clone();
        } else {
            // vec variant with sorting
            //self.late_scores.sort();
            //self.late_scores.reverse();
            //candidate_to_compare_score = self.late_scores[0].clone();

            //VecDeque variant
            candidate_to_compare_score = self.late_scores.back().unwrap().clone();
        }

        let mut new_population;
        let mut found_acceptable = false;
        let candidate_score = candidates[0].score.clone();
        if (candidate_score <= candidate_to_compare_score) || (candidate_score <= current_population[0].score) {
            found_acceptable = true;
            let best_candidate = candidates[0].clone();
            new_population = vec![best_candidate; 1];
            if current_population.len() > 1 {
                current_population[1..current_population.len()].iter().for_each(|individual| new_population.push(individual.clone()));
            }

            // vec variant with sorting
            //self.late_scores.push(candidate_score);

            //VecDeque variant
            self.late_scores.push_front(candidate_score);
            //println!("{:?}, {:?}", self.late_scores, candidate_to_compare_score);
            if self.late_scores.len() > self.late_acceptance_size {

                // vec variant with sorting
                //self.late_scores = self.late_scores[1..].to_vec();

                // VecDeque variant
                self.late_scores.pop_back();
            }
        } else {
            new_population = current_population.clone();
        }

        return (new_population, found_acceptable);
    }

    fn get_metaheuristic_kind(&self) -> MetaheuristicKind {
        self.metaheuristic_kind.clone()
    }

    fn get_metaheuristic_name(&self) -> MetaheuristicNames {
        self.metaheuristic_name.clone()
    }
}

impl<ScoreType> MoveTrait for LateAcceptanceBase<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {

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

unsafe impl<ScoreType> Send for LateAcceptanceBase<ScoreType>
where ScoreType: ScoreTrait + Clone + AddAssign + PartialEq + PartialOrd + Ord + Debug + Send {}